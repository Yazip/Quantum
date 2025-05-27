import 'package:flutter/material.dart';
import 'screens/login_screen.dart';

void main() {
  runApp(const QuantumApp());
}

class QuantumApp extends StatefulWidget {
  const QuantumApp({super.key});

  @override
  State<QuantumApp> createState() => _QuantumAppState();
}

class _QuantumAppState extends State<QuantumApp> {
  String? token;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Quantum Messenger',
      theme: ThemeData.dark(),
      home: token == null
          ? LoginScreen(onLogin: (receivedToken) {
              setState(() => token = receivedToken);
            })
          : Scaffold(
              body: Center(
                child: Text("Добро пожаловать в Quantum! JWT: $token"),
              ),
            ),
    );
  }
}