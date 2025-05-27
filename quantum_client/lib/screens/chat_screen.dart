import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:web_socket_channel/web_socket_channel.dart';

class ChatScreen extends StatefulWidget {
  final String chatTitle;
  final String chatId; // Добавим ID чата
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
  bool isAuthenticated = false;
  final TextEditingController _controller = TextEditingController();
  final List<String> _messages = [];
  late WebSocketChannel _channel;

  @override
  void initState() {
    super.initState();

    _channel = WebSocketChannel.connect(
      Uri.parse('ws://localhost:9001'),
    );

    // авторизация по токену
    _channel.sink.add(jsonEncode({
      "type": "auth",
      "token": widget.token,
    }));

    _channel.stream.listen((message) {
      print("Сервер прислал: $message");
      final data = jsonDecode(message);

      if (data["status"] == "authenticated") {
    	  setState(() => isAuthenticated = true);
	  print("Авторизация прошла");
      }

      if (data["status"] == "message_saved") {
        setState(() {
          _messages.add("[Вы] ${_controller.text}");
        });
        _controller.clear();
      }

      if (data["error"] != null) {
    	  print("Ошибка от сервера: ${data["error"]}");
      }
    },
    onError: (error) {
    	print("WebSocket ошибка: $error");
    },
    onDone: () {
    	print("WebSocket соединение закрыто");
    },);
  }

  void _sendMessage() {
    print("Попытка отправки...");
    if (!isAuthenticated) {
    	print("Не авторизован в WebSocket");
    	return;
    }

    final text = _controller.text.trim();
    if (text.isEmpty) return;

    print("Отправка: $text");

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
                return Align(
                  alignment: Alignment.centerRight,
                  child: Container(
                    margin: const EdgeInsets.symmetric(vertical: 4),
                    padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
                    decoration: BoxDecoration(
                      color: Theme.of(context).colorScheme.primary.withOpacity(0.2),
                      borderRadius: BorderRadius.circular(12),
                    ),
                    child: Text(msg),
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