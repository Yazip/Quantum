import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:web_socket_channel/web_socket_channel.dart';

class LoginScreen extends StatefulWidget {
  final Function(String token) onLogin;

  const LoginScreen({super.key, required this.onLogin});

  @override
  State<LoginScreen> createState() => _LoginScreenState();
}

class _LoginScreenState extends State<LoginScreen> {
  final _usernameController = TextEditingController();
  final _passwordController = TextEditingController();
  late WebSocketChannel _channel;
  String _error = "";

  @override
  void initState() {
    super.initState();
    _channel = WebSocketChannel.connect(
      Uri.parse('ws://192.168.0.101:9001'), // адрес сервера
    );

    _channel.stream.listen((message) {
      final data = jsonDecode(message);
      if (data["status"] == "ok" && data["token"] != null) {
        widget.onLogin(data["token"]);
      } else if (data["error"] != null) {
        setState(() => _error = data["error"]);
      }
    });
  }

  void _login() {
    final payload = {
      "type": "login",
      "payload": {
        "username": _usernameController.text,
        "password": _passwordController.text,
      }
    };
    _channel.sink.add(jsonEncode(payload));
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text("Quantum Login")),
      body: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          children: [
            TextField(
              controller: _usernameController,
              decoration: const InputDecoration(labelText: "Username"),
            ),
	    const SizedBox(height: 16),
            TextField(
              controller: _passwordController,
              decoration: const InputDecoration(labelText: "Password"),
              obscureText: true,
            ),
            const SizedBox(height: 24),
            ElevatedButton(onPressed: _login, child: const Text("Log In")),
            if (_error.isNotEmpty) ...[
              const SizedBox(height: 12),
              Text(_error, style: const TextStyle(color: Colors.red)),
            ]
          ],
        ),
      ),
    );
  }

  @override
  void dispose() {
    _channel.sink.close();
    super.dispose();
  }
}