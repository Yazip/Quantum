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

      if (data["status"] == "chat_exists") {
        final chatId = data["chat_id"];
        final chatTitle = data["chat_name"] ?? "Личный чат";;

        Navigator.push(
            context,
            MaterialPageRoute(
                builder: (_) => ChatScreen(
                    chatTitle: chatTitle,
                    chatId: chatId,
                    token: widget.token,
                ),
            ),
        );
      }

      if (data["status"] == "chat_created") {
    	  _channel.sink.add(jsonEncode({"type": "get_my_chats"}));
      }
    });

    // Авторизуемся
    _channel.sink.add(jsonEncode({
      "type": "auth",
      "token": widget.token,
    }));
  }

  void _showCreateChatDialog() {
      final nameController = TextEditingController();
      final participantsController = TextEditingController();
      String chatType = "group";

      showDialog(
    	  context: context,
    	  builder: (context) {
	      return StatefulBuilder(
	      	  builder: (context, setModalState) {
      	      	return AlertDialog(
                    title: const Text("Создание чата"),
                    content: Column(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                          TextField(
                                controller: nameController,
                                enabled: chatType != "private", // Блокируем при личном чате
                                decoration: InputDecoration(
                                    labelText: "Название чата",
                                    hintText: chatType == "private" ? "Название задаётся автоматически" : null,
                                ),
                          ),
                const SizedBox(height: 12),
                          TextField(
                                controller: participantsController,
                                decoration: const InputDecoration(
                              labelText: "Участники (через запятую)",
                                ),
                          ),
                          const SizedBox(height: 12),
                          DropdownButton<String>(
                                value: chatType,
                                isExpanded: true,
                                items: const [
                              DropdownMenuItem(value: "group", child: Text("Групповой")),
                              DropdownMenuItem(value: "private", child: Text("Личный")),
                                ],
                                onChanged: (value) {
                                    if (value != null) {
                                        setModalState(() {
                                            chatType = value;
                                            if (chatType == "private") {
                                                nameController.clear(); // очищаем название при выборе "Личный"
                                            }
                                    });
                              }
                                },
                          )
                          ],
        	),
        	actions: [
          	    TextButton(
            		onPressed: () => Navigator.pop(context),
            		child: const Text("Отмена"),
          	    ),
          	    ElevatedButton(
            		onPressed: () {
              		    final name = nameController.text.trim();
              		    final members = participantsController.text
                  		.split(',')
                  		.map((s) => s.trim())
                  		.where((s) => s.isNotEmpty)
                  		.toList();

                      // не передаём название если тип "private"
                      final chatName = chatType == "private" ? "" : name;

              		    if (chatType == "private" || name.isNotEmpty) {
                		_createChat(chatName, chatType, members);
                		Navigator.pop(context);
              		    }
            		},
            		child: const Text("Создать"),
          	    ),
          	],
      	  );
        },
      );  
    },
   );
  }

  void _createChat(String name, String type, List<String> participants) {
      final payload = {
    	  "type": "create_chat",
    	  "payload": {
      	      "name": name,
      	      "chat_type": type,
      	      "members": participants,
    	  }
      };

      _channel.sink.add(jsonEncode(payload));
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
      		icon: const Icon(Icons.add),
      		tooltip: "Создать чат",
      		onPressed: _showCreateChatDialog,
    	    ),
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