import 'package:flutter/material.dart';
import 'chat_screen.dart';

class ChatListScreen extends StatelessWidget {
  final String token;

  const ChatListScreen({super.key, required this.token});

  @override
  Widget build(BuildContext context) {
    final chats = [
      {"title": "ПИ-22", "lastMessage": "mx покажи йорика"},
      {"title": "Саня Питон", "lastMessage": "здарова"},
      {"title": "Строевая кошка", "lastMessage": "Шагом марш!"},
    ];

    return Scaffold(
      appBar: AppBar(
        title: const Text("Quantum — Чаты"),
        centerTitle: true,
      ),
      body: ListView.builder(
        padding: const EdgeInsets.all(12),
        itemCount: chats.length,
        itemBuilder: (context, index) {
          final chat = chats[index];
          return Card(
            margin: const EdgeInsets.symmetric(vertical: 8),
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(12),
            ),
            child: ListTile(
              title: Text(
                chat["title"]!,
                style: const TextStyle(fontSize: 18, fontWeight: FontWeight.w500),
              ),
              subtitle: Text(
                chat["lastMessage"]!,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
              ),
              onTap: () {
                Navigator.push(
    		    context,
    		    MaterialPageRoute(
      			builder: (_) => ChatScreen(
        		    chatTitle: chat["title"]!,
			    chatId: chat["id"] ?? "d50bfa99-dffa-4fd1-ab3f-b0a74fdaf249", // Временный ID
        		    token: token,
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