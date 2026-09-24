import 'package:clinicore/app/app.dart';
import 'package:clinicore/shared/env/env.dart';
import 'package:flutter/widgets.dart';

void main() {
  runApp(appFor(readConfiguration()));
}
