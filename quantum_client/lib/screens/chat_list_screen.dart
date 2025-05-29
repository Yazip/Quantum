import 'package:flutter/material.dart';
import 'package:web_socket_channel/web_socket_channel.dart';
import 'dart:convert';
import 'chat_screen.dart';

class ChatListScreen extends StatefulWidget {
  final String token;

  final void Function() onLogout;

  const ChatListScreen({Key? key, required this.token, required this.onLogout}) : super(key: key);

  @override
  State<ChatListScreen> createState() => _ChatListScreenState();
}

class _ChatListScreenState extends State<ChatListScreen> {
  late WebSocketChannel _channel;
  List<Map<String, dynamic>> _chats = [];

  @override
  void initState() {
    super.initState();

    _channel = WebSocketChannel.connect(Uri.parse('ws://192.168.0.101:9001'));

    _channel.stream.listen((event) {
      final data = jsonDecode(event);
      if (data["status"] == "authenticated") {
        _channel.sink.add(jsonEncode({"type": "get_my_chats"}));
      }

      if (data["type"] == "chat_list") {
        setState(() {
          _chats = List<Map<String, dynamic>>.from(data["chats"]);
        });
      }
    });

    // Авторизуемся
    _channel.sink.add(jsonEncode({
      "type": "auth",
      "token": widget.token,
    }));
  }

  @override
  void dispose() {
    _channel.sink.close();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text("Quantum — Чаты"),
        centerTitle: true,
	actions: [
    	    IconButton(
      		icon: const Icon(Icons.logout),
      		tooltip: "Выйти",
      		onPressed: widget.onLogout,
    	    ),
  	],
      ),
      body: ListView.builder(
        padding: const EdgeInsets.all(12),
        itemCount: _chats.length,
        itemBuilder: (context, index) {
          final chat = _chats[index];
          return Card(
            margin: const EdgeInsets.symmetric(vertical: 8),
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(12),
            ),
            child: ListTile(
              title: Text(
                chat["name"],
                style: const TextStyle(fontSize: 18, fontWeight: FontWeight.w500),
              ),
              subtitle: Text(chat["chat_type"]),
              onTap: () {
                Navigator.push(
                  context,
                  MaterialPageRoute(
                    builder: (_) => ChatScreen(
                      chatTitle: chat["name"],
                      chatId: chat["id"],
                      token: widget.token,
                    ),
                  ),
                );
              },
            ),
          );
        },
      ),
    );
  }
}