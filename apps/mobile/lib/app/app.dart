import 'package:clinicore/app/router.dart';
import 'package:clinicore/shared/env/env.dart';
import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

Widget appFor(ConfigurationResult result) {
  return switch (result) {
    MissingConfiguration(:final names) => MissingConfigurationApp(names: names),
    ValidConfiguration() => ClinicoreApp(),
  };
}

class ClinicoreApp extends StatelessWidget {
  new({super.key});

  final GoRouter _router = createRouter();

  @override
  Widget build(BuildContext context) {
    return MaterialApp.router(title: 'Clinicore', routerConfig: _router);
  }
}

class MissingConfigurationApp extends StatelessWidget {
  const new({required this.names, super.key});

  final List<String> names;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: Scaffold(
        body: SafeArea(
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Text('Missing configuration: ${names.join(', ')}'),
          ),
        ),
      ),
    );
  }
}
