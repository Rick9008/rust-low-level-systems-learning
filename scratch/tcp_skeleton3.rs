/*
1. TcpListener bind(記住 d#1 的教訓:測試場景用 :0 + local_addr() 拿真 port)
2. accept loop(incoming())
3. 每連線 thread::spawn
4. 連線內:read 進 buffer 的迴圈(讀到 0 = 對端關閉)
5. 回寫(write_all)
6. 必要的 use 塊——上兩輪你的傷疤有一半在這
*/

use std::net::TcpListener;
use std::io::{Read, Write};
use std::thread;

pub fn serve(tcp_listener: TcpListener) {
    'accept: loop {
        for stream in tcp_listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    thread::spawn(move || {
                        let mut buf = [0;4096];
                        loop {
                            match stream.read(&mut buf) {
                                Ok(0) => break,
                                Ok(n) => {
                                    if stream.write_all(&buf[0..n]).is_err() {
                                        break;
                                    }
                                },
                                Err(_) => break,
                            } 
                        }
                    });
                },
                Err(_) => break 'accept,
            }
        }
    }
}
