import 'package:flutter/material.dart';
import 'screens/login_screen.dart';
import 'theme/quantum_theme.dart';
import 'screens/chat_list_screen.dart';
import 'screens/register_screen.dart';

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
      debugShowCheckedModeBanner: false,
      theme: QuantumTheme.darkTheme,
      routes: {
    	  '/register': (context) => const RegisterScreen(),
      },
      home: token == null
          ? LoginScreen(onLogin: (receivedToken) {
              setState(() => token = receivedToken);
            })
          : Scaffold(
              body: Center(
                child: ChatListScreen(token: token!, onLogout: () => setState(() => token = null)),
              ),
            ),
    );
  }
}