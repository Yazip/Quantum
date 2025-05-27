import 'package:flutter/material.dart';

class QuantumTheme {
  static const Color primary = Color(0xFF9B59B6);      // Фиолетовый
  static const Color secondary = Color(0xFF8E44AD);    // Тёмно-фиолетовый
  static const Color background = Color(0xFF1C1C28);   // Тёмный графит
  static const Color surface = Color(0xFF2C2C3C);      // Поверхности
  static const Color textPrimary = Color(0xFFE0E0E0);  // Основной текст
  static const Color textSecondary = Color(0xFFAAAAAA);

  static ThemeData get darkTheme {
    return ThemeData(
      brightness: Brightness.dark,
      scaffoldBackgroundColor: background,
      primaryColor: primary,
      colorScheme: const ColorScheme.dark(
        primary: primary,
        secondary: secondary,
        background: background,
      ),
      textTheme: const TextTheme(
        bodyLarge: TextStyle(color: textPrimary, fontSize: 18),
        bodyMedium: TextStyle(color: textSecondary, fontSize: 14),
      ),
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: surface,
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(12),
        ),
        labelStyle: const TextStyle(color: textSecondary),
      ),
      elevatedButtonTheme: ElevatedButtonThemeData(
        style: ElevatedButton.styleFrom(
          backgroundColor: primary,
          foregroundColor: Colors.white,
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(14),
          ),
          padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 14),
          textStyle: const TextStyle(fontSize: 16),
        ),
      ),
      cardColor: surface,
    );
  }
}