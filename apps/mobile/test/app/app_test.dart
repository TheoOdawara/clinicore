import 'package:clinicore/app/app.dart';
import 'package:clinicore/shared/env/env.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  testWidgets('shows the missing names and never reaches the app', (
    tester,
  ) async {
    final result = parseConfiguration({
      'GOOGLE_SERVER_CLIENT_ID': 'server.apps.googleusercontent.com',
      'GOOGLE_IOS_CLIENT_ID': 'ios.apps.googleusercontent.com',
    });

    await tester.pumpWidget(appFor(result));

    expect(find.text('Missing configuration: API_URL'), findsOneWidget);
    expect(find.text('Clinicore'), findsNothing);
  });

  testWidgets('opens / when the configuration is complete', (tester) async {
    final result = parseConfiguration({
      'API_URL': 'https://api.clinicore.com.br',
      'GOOGLE_SERVER_CLIENT_ID': 'server.apps.googleusercontent.com',
      'GOOGLE_IOS_CLIENT_ID': 'ios.apps.googleusercontent.com',
    });

    await tester.pumpWidget(appFor(result));
    await tester.pumpAndSettle();

    expect(find.text('Clinicore'), findsOneWidget);
  });
}
