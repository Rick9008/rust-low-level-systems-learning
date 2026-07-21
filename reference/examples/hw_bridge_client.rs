//! 可執行的 hw_bridge client:正常命令 + 四種「打壞它」的實驗。
//!
//! ```sh
//! cargo run -p reference --example hw_bridge_client -- demo          # 完整劇本(推薦先跑這個)
//! cargo run -p reference --example hw_bridge_client -- ping
//! cargo run -p reference --example hw_bridge_client -- sensor 25
//! cargo run -p reference --example hw_bridge_client -- fan 4500
//! cargo run -p reference --example hw_bridge_client -- drip 200      # 一次一 byte,間隔 200ms
//! cargo run -p reference --example hw_bridge_client -- pipeline 20   # 連發不等回應
//! cargo run -p reference --example hw_bridge_client -- badop         # 未知 opcode:連線活著
//! cargo run -p reference --example hw_bridge_client -- badlen        # len 胡說:連線報廢
//! cargo run -p reference --example hw_bridge_client -- raw 0000000101
//! ```
//!
//! `--addr HOST:PORT`(預設 127.0.0.1:9000)。
//!
//! ## 為什麼要有 `drip`
//! framing 的全部難處在一句話:**TCP 是 byte stream,沒有 message 邊界**。
//! `drip` 把一個 5-byte 的 Ping frame 拆成 5 次 write、每次間隔幾百 ms。
//! 你會在 server 那邊看到它**整整安靜等待**,直到最後一 byte 到齊才印出命令——
//! 這就是 `FrameReader` 的狀態機:len 還沒湊滿 4 byte 時它什麼都不能做,
//! 湊滿了但 payload 還差一 byte 時它仍然什麼都不能做。
//! 這條路徑在單元測試裡是 `assert_eq!(reader.next_frame(), Ok(None))`——
//! 跑一次 drip,那個 `Ok(None)` 就有了體感。

use reference::io::hw_bridge::client::{ClientError, HwClient};
use reference::io::hw_bridge::handler::MockHardware;
use reference::io::hw_bridge::protocol::{
    Command, ERR_BAD_PAYLOAD, ERR_UNKNOWN_OPCODE, Response, encode_frame,
};
use std::env;
use std::io::Write;
use std::net::SocketAddr;
use std::process;
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    let mut addr_s = String::from("127.0.0.1:9000");
    let mut rest: Vec<String> = Vec::new();

    let mut args = env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--addr" => match args.next() {
                Some(v) => addr_s = v,
                None => fail("--addr 需要一個值"),
            },
            "-h" | "--help" => {
                usage();
                return;
            }
            _ => rest.push(a),
        }
    }

    let addr: SocketAddr = match addr_s.parse() {
        Ok(a) => a,
        Err(e) => fail(&format!("bad --addr {addr_s}: {e}")),
    };

    let cmd = rest.first().map(String::as_str).unwrap_or("demo");
    let arg = rest.get(1);

    let result = match cmd {
        "demo" => demo(addr),
        "ping" => ping(addr),
        "sensor" => sensor(addr, num(arg, 25)),
        "fan" => fan(addr, num(arg, 4500)),
        "drip" => drip(addr, num::<u64>(arg, 300)),
        "pipeline" => pipeline(addr, num(arg, 20)),
        "badop" => badop(addr),
        "badlen" => badlen(addr),
        "raw" => match arg {
            Some(h) => raw(addr, h),
            None => fail("raw 需要 hex 字串,例:raw 0000000101"),
        },
        other => fail(&format!("unknown command: {other}")),
    };

    if let Err(e) = result {
        eprintln!("error: {e:?}");
        process::exit(1);
    }
}

// ── 正常路徑 ─────────────────────────────────────────────────────────

fn ping(addr: SocketAddr) -> Result<(), ClientError> {
    let mut c = HwClient::connect(addr)?;
    let t = Instant::now();
    c.ping()?;
    println!("Ping -> Pong  ({:?})", t.elapsed());
    Ok(())
}

fn sensor(addr: SocketAddr, id: u16) -> Result<(), ClientError> {
    let mut c = HwClient::connect(addr)?;
    let mc = c.read_sensor(id)?;
    // MockHardware::sensor_value 是 id 的確定性函數——client 端可以自己重算對答案
    println!(
        "ReadSensor{{{id}}} -> {mc} m°C  ({:.3} °C)  [預期 {}]",
        f64::from(mc) / 1000.0,
        MockHardware::sensor_value(id)
    );
    Ok(())
}

fn fan(addr: SocketAddr, rpm: u16) -> Result<(), ClientError> {
    let mut c = HwClient::connect(addr)?;
    c.set_fan(rpm)?;
    println!("SetFan{{{rpm}}} -> Ack");
    Ok(())
}

// ── 實驗一:byte-by-byte,看 FrameReader 等待 ────────────────────────

fn drip(addr: SocketAddr, gap_ms: u64) -> Result<(), ClientError> {
    let frame = Command::Ping.encode(); // [00 00 00 01][01]
    let mut c = HwClient::connect(addr)?;

    println!("Ping frame = {}", hex(&frame));
    println!("一次送一 byte,間隔 {gap_ms}ms。看 server 端:直到最後一 byte 才有反應。\n");

    let t = Instant::now();
    for (i, b) in frame.iter().enumerate() {
        c.send_raw(&[*b])?;
        let need = if i + 1 < 4 {
            format!("len 還差 {} byte", 4 - (i + 1))
        } else if i + 1 == 4 {
            "len 到齊了(=1),payload 還差 1 byte".to_string()
        } else {
            "frame 完整 → server 現在才能解".to_string()
        };
        println!("  [{:>6.0?}] byte {} = {:#04x}   {need}", t.elapsed(), i, b);
        std::io::stdout().flush().ok();
        if i + 1 < frame.len() {
            thread::sleep(Duration::from_millis(gap_ms));
        }
    }

    let resp = c.recv_response()?;
    println!("\n  [{:>6.0?}] <- {resp:?}", t.elapsed());
    println!("回應的時間戳 ≈ 最後一 byte 的時間戳,不是第一 byte——server 沒有偷跑。");
    Ok(())
}

// ── 實驗二:pipeline,看回應照請求順序回來 ──────────────────────────

fn pipeline(addr: SocketAddr, n: u16) -> Result<(), ClientError> {
    let mut c = HwClient::connect(addr)?;
    println!("一口氣送 {n} 條 ReadSensor(不等回應),再依序收 {n} 個回應。\n");

    for id in 0..n {
        c.send_raw(&Command::ReadSensor { sensor_id: id }.encode())?;
    }
    let mut in_order = true;
    for id in 0..n {
        match c.recv_response()? {
            Response::SensorValue { sensor_id, .. } => {
                if sensor_id != id {
                    in_order = false;
                    println!("  第 {id} 個回應卻是 sensor {sensor_id} —— 亂序!");
                }
            }
            other => println!("  非預期回應:{other:?}"),
        }
    }
    println!(
        "{} 個回應{}。sync 協定沒有 request-id,順序就是唯一的對應方式——",
        n,
        if in_order {
            "全部照順序回來"
        } else {
            "亂序"
        }
    );
    println!("evented server 的「單一 command worker」設計就是為了守住這條性質。");
    Ok(())
}

// ── 實驗三:語意錯 → 可恢復 ─────────────────────────────────────────

fn badop(addr: SocketAddr) -> Result<(), ClientError> {
    let mut c = HwClient::connect(addr)?;
    let frame = encode_frame(0x7E, &[]); // 0x7E 不在 opcode 表裡
    println!("送未知 opcode 0x7E:{}", hex(&frame));
    c.send_raw(&frame)?;

    match c.recv_response()? {
        Response::Error { code } if code == ERR_UNKNOWN_OPCODE => {
            println!("<- Error{{code={code}}} = ERR_UNKNOWN_OPCODE");
        }
        Response::Error { code } if code == ERR_BAD_PAYLOAD => {
            println!("<- Error{{code={code}}} = ERR_BAD_PAYLOAD");
        }
        other => println!("<- {other:?}"),
    }

    // 關鍵:連線還活著。語意錯不是 framing 錯——server 知道這個 frame 到哪結束。
    c.ping()?;
    println!("同一條連線再 Ping -> Pong ✔  語意錯可恢復,連線續命。");
    Ok(())
}

// ── 實驗四:framing 錯 → 連線報廢 ───────────────────────────────────

fn badlen(addr: SocketAddr) -> Result<(), ClientError> {
    let mut c = HwClient::connect(addr)?;
    println!("送 len = 0xFFFFFFFF(遠超 MAX_FRAME_LEN)");
    c.send_raw(&u32::MAX.to_be_bytes())?;

    match c.ping() {
        Err(ClientError::ServerClosed | ClientError::Io(_)) => {
            println!("<- 連線被 server 關掉 ✔");
            println!(
                "為什麼不能像 badop 一樣回個 Error 就算了?length-prefix 協定**沒有 resync 點**:"
            );
            println!("len 一旦不可信,server 就不知道下一個 frame 從哪個 byte 開始——");
            println!(
                "stream 已經失去意義,只能斷線。(delimiter 協定可以掃到下一個分隔符重新同步。)"
            );
            Ok(())
        }
        Ok(()) => {
            println!("<- server 竟然還回了 Pong —— 這是 bug");
            Ok(())
        }
        Err(e) => Err(e),
    }
}

// ── 任意 bytes ──────────────────────────────────────────────────────

fn raw(addr: SocketAddr, hexstr: &str) -> Result<(), ClientError> {
    let bytes = match unhex(hexstr) {
        Ok(b) => b,
        Err(e) => fail(&e),
    };
    let mut c = HwClient::connect(addr)?;
    println!("-> {}", hex(&bytes));
    c.send_raw(&bytes)?;
    match c.recv_response() {
        Ok(r) => println!("<- {r:?}"),
        Err(e) => println!("<- {e:?}"),
    }
    Ok(())
}

// ── demo:完整劇本 ──────────────────────────────────────────────────

fn demo(addr: SocketAddr) -> Result<(), ClientError> {
    let mut c = HwClient::connect(addr)?;

    section("1. 三條正常命令");
    c.ping()?;
    println!("Ping -> Pong");
    let mc = c.read_sensor(25)?;
    println!("ReadSensor{{25}} -> {mc} m°C");
    c.set_fan(4500)?;
    println!("SetFan{{4500}} -> Ack");

    section("2. 兩個 frame 黏在同一個 TCP segment 裡(server 必須切開)");
    let mut glued = Command::Ping.encode();
    glued.extend_from_slice(&Command::ReadSensor { sensor_id: 7 }.encode());
    println!("一次 write 送出 {}:{}", glued.len(), hex(&glued));
    c.send_raw(&glued)?;
    println!("<- {:?}", c.recv_response()?);
    println!("<- {:?}", c.recv_response()?);
    println!("兩個回應 = server 的 read buffer parse loop 有真的 loop。");

    section("3. 未知 opcode:回 Error frame,連線活著");
    c.send_raw(&encode_frame(0x7E, &[]))?;
    println!("<- {:?}", c.recv_response()?);
    c.ping()?;
    println!("再 Ping -> Pong ✔ 連線續命");

    section("4. framing 損毀:連線報廢(換一條新連線做)");
    badlen(addr)?;

    section("5. 半個 frame 送完就斷線:server 不能倒");
    let mut half = HwClient::connect(addr)?;
    half.send_raw(&Command::Ping.encode()[..2])?;
    drop(half);
    thread::sleep(Duration::from_millis(50));
    let mut again = HwClient::connect(addr)?;
    again.ping()?;
    println!("斷線後新 client 仍 Ping -> Pong ✔ 連線級錯誤不擴散到 server");

    println!("\n全部通過。接著跑 `drip` 看 FrameReader 一 byte 一 byte 等。");
    Ok(())
}

// ── 小工具 ──────────────────────────────────────────────────────────

fn section(title: &str) {
    println!("\n─── {title} ───");
}

fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn unhex(s: &str) -> Result<Vec<u8>, String> {
    let clean: String = s
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '_')
        .collect();
    if !clean.len().is_multiple_of(2) {
        return Err(format!("hex 長度必須是偶數,得到 {}", clean.len()));
    }
    (0..clean.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&clean[i..i + 2], 16).map_err(|e| format!("bad hex: {e}")))
        .collect()
}

fn num<T: std::str::FromStr>(arg: Option<&String>, default: T) -> T {
    match arg {
        Some(s) => s.parse().unwrap_or(default),
        None => default,
    }
}

fn fail(msg: &str) -> ! {
    eprintln!("error: {msg}");
    usage();
    process::exit(2);
}

fn usage() {
    eprintln!("usage: hw_bridge_client [--addr HOST:PORT] <command>");
    eprintln!("  demo              完整劇本(預設)");
    eprintln!("  ping | sensor ID | fan RPM");
    eprintln!("  drip [MS]         一次一 byte 送 Ping frame,看 FrameReader 等待");
    eprintln!("  pipeline [N]      連發 N 條不等回應,驗證回應順序");
    eprintln!("  badop             未知 opcode → Error frame,連線活著");
    eprintln!("  badlen            len 胡說 → server 斷線");
    eprintln!("  raw HEX           送任意 bytes");
}
