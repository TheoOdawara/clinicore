import 'package:clinicore/app/router.dart';
import 'package:clinicore/shared/env/env.dart';
import 'package:flutter/material.dart';

Widget appFor(ConfigurationResult result) {
  return switch (result) {
    MissingConfiguration(:final names) => MissingConfigurationApp(names: names),
    ValidConfiguration() => ClinicoreApp(),
  };
}

class ClinicoreApp extends StatelessWidget {
  ClinicoreApp({super.key});

  final _router = createRouter();

  @override
  Widget build(BuildContext context) {
    return MaterialApp.router(title: 'Clinicore', routerConfig: _router);
  }
}

class MissingConfigurationApp extends StatelessWidget {
  const MissingConfigurationApp({super.key, required this.names});

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
