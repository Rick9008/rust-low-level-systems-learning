// 8/5 晚 mock 面試參考解:Dependency-aware Job Scheduler(C++17, std-only)
// 編譯:g++ -std=c++17 -Wall -Wextra -pthread mock_cpp_0805_sol.cpp -o sched && ./sched
// 面試官用:對照 candidate 的設計;不要在 mock 中展示。
//
// 設計要點(對應題目包 §經典bug清單):
//  - 每個 job 一個 remaining 計數 + dependents 鄰接表(= Kahn 的 in-degree 化身)
//  - dep 已完成才 submit 的 job:查 completed_ 集合,視為已滿足(經典卡死點)
//  - finish() 全程持鎖:銷帳(erase)與放行(push ready)同一臨界區,無競態窗
//  - 跑 user fn 時「不持鎖」(否則單線程化 + 重入死鎖)
//  - shutdown = drain:等 nodes_ 清空才放 worker 走;nodes_ 含 waiting/ready/running
//  - remove-at-zero:job 完成即 erase,live map 大小 ≤ 未完成數(不無限長)
//    trade-off:completed_ 集合仍單調成長,見題目包 follow-up 3

#include <atomic>
#include <cassert>
#include <chrono>
#include <condition_variable>
#include <cstdint>
#include <cstdio>
#include <deque>
#include <functional>
#include <mutex>
#include <stdexcept>
#include <thread>
#include <unordered_map>
#include <unordered_set>
#include <vector>

class JobScheduler {
public:
    using JobId = std::uint64_t;

    explicit JobScheduler(std::size_t n_workers) {
        workers_.reserve(n_workers);
        for (std::size_t i = 0; i < n_workers; ++i)
            workers_.emplace_back([this] { worker_loop(); });
    }

    // deps 必須是先前 submit() 回傳的 id(⇒ 圖必為 DAG,環在建構上不可能)
    JobId submit(std::function<void()> fn, const std::vector<JobId>& deps = {}) {
        std::lock_guard<std::mutex> lk(mu_);
        if (shutting_down_) throw std::runtime_error("submit after shutdown");

        const JobId id = next_id_++;
        Node node;
        node.fn = std::move(fn);
        for (JobId d : deps) {
            auto it = nodes_.find(d);
            if (it != nodes_.end()) {          // dep 還活著(waiting/ready/running)
                it->second.dependents.push_back(id);
                ++node.remaining;
            } else if (!completed_.count(d)) { // 不活著也沒完成過 = 假 id
                throw std::invalid_argument("unknown dep id");
            }                                  // else:dep 已完成,視為已滿足
        }
        const bool ready = (node.remaining == 0);
        nodes_.emplace(id, std::move(node));
        if (ready) {
            ready_.push_back(id);
            cv_.notify_one();
        }
        return id;
    }

    // drain 語意:已 submit 的全部跑完才停;之後 submit 會丟例外
    void shutdown() {
        {
            std::lock_guard<std::mutex> lk(mu_);
            shutting_down_ = true;   // 先立 flag(持鎖)……
        }
        cv_.notify_all();            // ……再叫醒全部(順序反了 = 丟失喚醒)
        for (auto& t : workers_) t.join();
        workers_.clear();
    }

    ~JobScheduler() {
        if (!workers_.empty()) shutdown();
    }

private:
    struct Node {
        std::function<void()> fn;
        std::size_t remaining = 0;        // 未完成的 dep 數(Kahn in-degree)
        std::vector<JobId> dependents;    // 完成時要通知誰(鄰接表)
    };

    void worker_loop() {
        std::unique_lock<std::mutex> lk(mu_);
        for (;;) {
            // nodes_ 空 ⇔ 沒有 waiting/ready/running 任何 job(running 也還在 nodes_ 裡)
            cv_.wait(lk, [this] { return !ready_.empty() || (shutting_down_ && nodes_.empty()); });
            if (ready_.empty()) return;   // 只可能是 drain 完成
            const JobId id = ready_.front();
            ready_.pop_front();
            auto fn = std::move(nodes_.at(id).fn);
            lk.unlock();
            fn();                         // user 程式碼不持鎖跑
            lk.lock();
            finish(id);
        }
    }

    // 前置:持有 mu_。銷帳與放行同一臨界區。
    void finish(JobId id) {
        auto it = nodes_.find(id);
        for (JobId child : it->second.dependents) {
            Node& c = nodes_.at(child);
            if (--c.remaining == 0) {
                ready_.push_back(child);
                cv_.notify_one();
            }
        }
        nodes_.erase(it);                 // remove-at-zero
        completed_.insert(id);
        if (shutting_down_ && nodes_.empty()) cv_.notify_all();  // 放 drain 等待者走
    }

    std::mutex mu_;
    std::condition_variable cv_;
    std::unordered_map<JobId, Node> nodes_;  // 未完成的 job(含 running)
    std::unordered_set<JobId> completed_;    // 已完成 id(驗證晚到的 dep)
    std::deque<JobId> ready_;
    std::vector<std::thread> workers_;
    JobId next_id_ = 1;
    bool shutting_down_ = false;
};

// ---------------------------------------------------------------- smoke tests

int main() {
    using namespace std::chrono_literals;

    // 1. 菱形依賴 A→(B,C)→D:A 最先、D 最後
    {
        JobScheduler s(4);
        std::mutex mu;
        std::vector<char> order;
        auto rec = [&](char c) { std::lock_guard<std::mutex> lk(mu); order.push_back(c); };
        auto a = s.submit([&] { rec('A'); });
        auto b = s.submit([&] { rec('B'); }, {a});
        auto c = s.submit([&] { rec('C'); }, {a});
        s.submit([&] { rec('D'); }, {b, c});
        s.shutdown();
        assert(order.size() == 4);
        assert(order.front() == 'A' && order.back() == 'D');
        std::printf("test1 diamond: ok (%c%c%c%c)\n", order[0], order[1], order[2], order[3]);
    }

    // 2. 經典卡死點:dep 在 submit 前就已完成 → job 仍必須跑
    {
        JobScheduler s(2);
        std::atomic<bool> a_done{false}, b_ran{false};
        auto a = s.submit([&] { a_done = true; });
        while (!a_done) std::this_thread::sleep_for(1ms);
        std::this_thread::sleep_for(10ms);            // 確保 A 的 finish() 已走完
        s.submit([&] { b_ran = true; }, {a});
        s.shutdown();
        assert(b_ran);
        std::printf("test2 late-dep: ok\n");
    }

    // 3. 平行度:4 顆 worker 跑 4 個 100ms 獨立 job,總時 ≪ 400ms
    {
        JobScheduler s(4);
        const auto t0 = std::chrono::steady_clock::now();
        for (int i = 0; i < 4; ++i) s.submit([] { std::this_thread::sleep_for(100ms); });
        s.shutdown();
        const auto ms =
            std::chrono::duration_cast<std::chrono::milliseconds>(std::chrono::steady_clock::now() - t0)
                .count();
        assert(ms < 300);
        std::printf("test3 parallelism: ok (%lldms)\n", static_cast<long long>(ms));
    }

    // 4. drain:submit 長鏈後立刻 shutdown,全部要跑完
    {
        std::atomic<int> ran{0};
        JobScheduler s(2);
        JobScheduler::JobId prev = 0;
        for (int i = 0; i < 50; ++i) {
            std::vector<JobScheduler::JobId> deps;
            if (i > 0) deps.push_back(prev);
            prev = s.submit([&] { ++ran; }, deps);
        }
        s.shutdown();
        assert(ran == 50);
        std::printf("test4 drain-chain: ok (%d)\n", ran.load());
    }

    std::printf("all smoke tests passed\n");
    return 0;
}
