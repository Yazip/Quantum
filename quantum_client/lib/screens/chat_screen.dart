import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:web_socket_channel/web_socket_channel.dart';
import 'package:jwt_decoder/jwt_decoder.dart';

class ChatScreen extends StatefulWidget {
  final String chatTitle;
  final String chatId;
  final String token;

  const ChatScreen({
    super.key,
    required this.chatTitle,
    required this.chatId,
    required this.token,
  });

  @override
  State<ChatScreen> createState() => _ChatScreenState();
}

class _ChatScreenState extends State<ChatScreen> {
  late String currentUsername;
  final TextEditingController _controller = TextEditingController();
  List<Map<String, dynamic>> _messages = [];
  late WebSocketChannel _channel;
  bool isAuthenticated = false;

  @override
  void initState() {
    super.initState();

    // Извлекаем имя пользователя из JWT
    Map<String, dynamic> decoded = JwtDecoder.decode(widget.token);

    _channel = WebSocketChannel.connect(
      Uri.parse('ws://192.168.0.101:9001'),
    );

    _channel.sink.add(jsonEncode({
      "type": "auth",
      "token": widget.token,
    }));

    _channel.sink.add(jsonEncode({
      "type": "get_messages",
      "payload": {
        "chat_id": widget.chatId,
      }
    }));

    _channel.stream.listen((message) {
      final data = jsonDecode(message);
      print("Сервер прислал: $data");

      if (data["status"] == "authenticated") {
        setState(() {
    	    isAuthenticated = true;
    	    currentUsername = data["username"];
  	});
        print("Авторизация прошла");
      }

      if (data["status"] == "message_saved") {
        _controller.clear();
      }

      if (data["type"] == "new_message") {
        final from = data["from"];
        final body = data["body"];
        final chatIdFromServer = data["chat_id"];

        if (chatIdFromServer == widget.chatId) {
          setState(() {
            _messages.add({"from": from, "body": body});
          });
        }
      }

      if (data["type"] == "message_history") {
        final messages = data["messages"] as List;
        setState(() {
          _messages = messages.map((m) => {
            "from": m["from"],
            "body": m["body"],
          }).toList();
        });
      }

      if (data["error"] != null) {
        print("Ошибка от сервера: ${data["error"]}");
      }
    }, onError: (error) {
      print("WebSocket ошибка: $error");
    }, onDone: () {
      print("WebSocket соединение закрыто");
    });
  }

  void _sendMessage() {
    if (!isAuthenticated) {
      print("Не авторизован в WebSocket");
      return;
    }

    final text = _controller.text.trim();
    if (text.isEmpty) return;

    final payload = {
      "type": "send_message",
      "payload": {
        "chat_id": widget.chatId,
        "body": text,
      }
    };

    _channel.sink.add(jsonEncode(payload));
  }

  @override
  void dispose() {
    _channel.sink.close();
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(widget.chatTitle),
      ),
      body: Column(
        children: [
          Expanded(
            child: ListView.builder(
              padding: const EdgeInsets.all(12),
              itemCount: _messages.length,
              itemBuilder: (context, index) {
                final msg = _messages[index];
                final from = msg["from"] ?? "???";
                final body = msg["body"] ?? "";

                final isMine = from == currentUsername;

                return Align(
                  alignment: isMine ? Alignment.centerRight : Alignment.centerLeft,
                  child: Container(
                    margin: const EdgeInsets.symmetric(vertical: 6),
                    padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                    constraints: const BoxConstraints(maxWidth: 300),
                    decoration: BoxDecoration(
                      color: isMine ? Colors.deepPurple[300] : Colors.grey[300],
                      borderRadius: BorderRadius.only(
                        topLeft: const Radius.circular(16),
                        topRight: const Radius.circular(16),
                        bottomLeft: Radius.circular(isMine ? 16 : 0),
                        bottomRight: Radius.circular(isMine ? 0 : 16),
                      ),
                    ),
                    child: Column(
                      crossAxisAlignment:
                          isMine ? CrossAxisAlignment.end : CrossAxisAlignment.start,
                      children: [
                        if (!isMine)
                          Text(
                            from,
                            style: TextStyle(
                              fontSize: 12,
                              fontWeight: FontWeight.bold,
                              color: Colors.grey[700],
                            ),
                          ),
                        Text(
                          body,
                          style: const TextStyle(fontSize: 16),
                        ),
                      ],
                    ),
                  ),
                );
              },
            ),
          ),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
            child: Row(
              children: [
                Expanded(
                  child: TextField(
                    controller: _controller,
                    decoration: const InputDecoration(
                      hintText: "Написать сообщение...",
                      border: OutlineInputBorder(),
                      isDense: true,
                    ),
                  ),
                ),
                const SizedBox(width: 10),
                ElevatedButton(
                  onPressed: _sendMessage,
                  child: const Text("Отправить"),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}