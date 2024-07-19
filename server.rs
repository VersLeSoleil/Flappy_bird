use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

fn handle_client(mut stream: TcpStream, scores: Arc<Mutex<Vec<i32>>>, results: Arc<Mutex<Vec<String>>>) {
    
    let mut buffer = [0; 4];
    // 读取客户端ID
    if let Err(e) = stream.read_exact(&mut buffer) {
        eprintln!("Failed to read client ID from stream: {}", e);
        return;
    }
    let client_id = u32::from_be_bytes(buffer);
    println!("Received client ID: {}", client_id);

    // 读取分数
    if let Err(e) = stream.read_exact(&mut buffer) {
        eprintln!("Failed to read score from stream for client {}: {}", client_id, e);
        return;
    }
    let score = i32::from_be_bytes(buffer);
    println!("Received score from client {}: {}", client_id, score);

    {
        let mut scores = scores.lock().unwrap();
        if client_id as usize >= scores.len() {
            eprintln!("Invalid client ID: {}", client_id);
            return;
        }
        scores[client_id as usize] = score;
    }

    // 循环等待所有客户端都已发送分数
    loop {
        let all_scores;
        {
            let scores = scores.lock().unwrap();
            all_scores = scores.clone();
        }

        if all_scores.iter().all(|&s| s != -1) {
            // 计算结果
            let player1_score = all_scores[0];
            let player2_score = all_scores[1];

            let result_message = if player1_score > player2_score {
                "Player 1 wins!".to_string()
            } else if player1_score < player2_score {
                "Player 2 wins!".to_string()
            } else {
                "It's a tie!".to_string()
            };

            // 记录每个玩家的结果信息
            {
                let mut results = results.lock().unwrap();
                results[0] = result_message.clone(); // Player 1
                results[1] = result_message.clone(); // Player 2
            }

            // 发送结果给客户端
            if let Err(e) = stream.write(result_message.as_bytes()) {
                eprintln!("Failed to write to stream for client {}: {}", client_id, e);
            }
            if let Err(e) = stream.flush() {
                eprintln!("Failed to flush stream for client {}: {}", client_id, e);
            }
            if let Err(e) = stream.shutdown(std::net::Shutdown::Both) {
                eprintln!("Failed to shutdown stream for client {}: {}", client_id, e);
            }
            break;
        }
    }
}

fn main() {
    let listener = TcpListener::bind("192.168.3.10:7878").unwrap();
    let scores = Arc::new(Mutex::new(vec![-1, -1])); // 初始化分数为-1，表示未发送
    let results = Arc::new(Mutex::new(vec![String::new(), String::new()])); // 初始化结果为空字符串

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let scores = Arc::clone(&scores);
                let results = Arc::clone(&results);
                thread::spawn(move || {
                    handle_client(stream, scores, results);
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}
