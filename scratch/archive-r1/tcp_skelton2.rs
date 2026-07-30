//! TCP 骨架默寫 rep#1 批改紀錄 —— 2026-07-25 咖啡廳(6 輪,7 洞 → 0)
//! 原版:`rehearsals/examples/tcp_skeleton_std.rs`;下一次:7/26 d-std 前 5m 重默,驗下面五條傷疤。
//!
//! ## 傷疤清單(重默驗收標準:一次寫對這五條)
//! 1. **拆殼固定用 let-else**:`let Ok(mut stream) = stream else { continue };`
//!    ——三輪寫了三種形式(跳過/if-let-else/混入 `=>`)= 沒有肌肉只有即興。一個形狀,零變體。
//! 2. **echo 逐次回寫在 loop 內**(`Ok(n)` 分支裡就 write)。累積 len 到 EOF 才寫 =
//!    client 等回音才關連線 → 互等死鎖;len 破 4096 → 越界 panic。
//! 3. **`Err(_) => break`**——`continue` 是壞 socket 無限重試。
//! 4. **`write_all`,永遠 `write_all`**(`write` 可能只寫一半)。本傷疤三次得而復失(R2 對→R3 掉→R4 對→R5 掉)。
//!    Result 不裸丟:`if ...is_err() { break }`。
//! 5. **slice 是 `&buf[..n]`**(R3 寫過 `[0;len]`——那是陣列初始化語法)。
//!
//! ## 已癒合(R1/R2 犯、R3 起沒再犯)
//! - buf 要**擁有的陣列** `[0u8; 4096]`,`&mut` 呼叫時才加(R1/R2 連犯 `&[0; 4096]` 引用版)。
//! - `mut stream`(read/write_all 都拿 `&mut self`)。
//! - `thread::spawn(move ||)` + loop + `Ok(0)`=EOF 三條肌肉 R3 起長出。
//!
//! ## 逐輪帳
//! R1:7 洞(Result 沒拆/無 spawn/buf 引用/read 沒 match/無 Ok(0)/無 loop/write)
//! R2:5 洞(buf 引用重犯/stream 沒 mut/`return;,` 語法/無 loop/無 Ok(0))
//! R3:5 洞(if-let 混 `=>`/`&mut` 掉 buf/**累積後一次寫=語意 bug**/Err=>continue/write 回歸)
//! R4:2 洞(echo 仍在 loop 外、Err=>continue——**皆為上輪點名過的**)
//! R5:1 洞(write_all 第三次掉)|R6:✓ 同構參考版
//!
//! ## meta 教訓
//! 錯誤模式不是「不會」,是**每輪即興重組**——上輪對的這輪丟。默寫的目的就是把即興換成
//! 固定形狀:`let-else 拆殼 → spawn(move) → loop { match read: Ok(0)=>break /
//! Ok(n)=>write_all(&buf[..n]) 失敗 break / Err=>break }`。
//!
//! 殘留風味(非錯):`TcpLoop` 該叫 `serve`(non-snake-case warning);if-let-else 五行可換 let-else 一行。

use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

fn TcpLoop(listener: TcpListener) {
    for stream in listener.incoming() {
        let mut stream = if let Ok(mut stream) = stream {
            stream
        } else {
            continue;
        };

        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match stream.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if stream.write_all(&buf[0..n]).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }
}
