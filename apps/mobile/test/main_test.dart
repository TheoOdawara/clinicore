import 'package:clinicore/main.dart' as main_entry;
import 'package:flutter_test/flutter_test.dart';

void main() {
  testWidgets('boots into the missing configuration screen without defines', (
    tester,
  ) async {
    main_entry.main();
    await tester.pump();

    expect(
      find.text(
        'Missing configuration: '
        'API_URL, GOOGLE_SERVER_CLIENT_ID, GOOGLE_IOS_CLIENT_ID',
      ),
      findsOneWidget,
    );
  });
}
