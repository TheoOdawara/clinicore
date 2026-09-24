import 'package:clinicore/shared/env/env.dart';
import 'package:flutter_test/flutter_test.dart';

const complete = {
  'API_URL': 'https://api.clinicore.com.br',
  'GOOGLE_SERVER_CLIENT_ID': 'server.apps.googleusercontent.com',
  'GOOGLE_IOS_CLIENT_ID': 'ios.apps.googleusercontent.com',
};

List<String> missingIn(ConfigurationResult result) {
  return switch (result) {
    MissingConfiguration(:final names) => names,
    ValidConfiguration() => const [],
  };
}

void main() {
  test('returns the configuration when every value is present', () {
    final result = parseConfiguration(complete);

    expect(result, isA<ValidConfiguration>());
    final configuration = (result as ValidConfiguration).configuration;
    expect(configuration.apiUrl, Uri.parse('https://api.clinicore.com.br'));
    expect(
      configuration.googleServerClientId,
      'server.apps.googleusercontent.com',
    );
    expect(configuration.googleIosClientId, 'ios.apps.googleusercontent.com');
  });

  test('lists API_URL when it is absent', () {
    final result = parseConfiguration({...complete}..remove('API_URL'));

    expect(missingIn(result), ['API_URL']);
  });

  for (final malformed in [
    '',
    'https://api.clinicore.com.br/',
    'api.clinicore.com.br',
    '/api',
  ]) {
    test('lists API_URL when it is "$malformed"', () {
      final result = parseConfiguration({...complete, 'API_URL': malformed});

      expect(missingIn(result), ['API_URL']);
    });
  }

  test('lists every missing name in order', () {
    final result = parseConfiguration({
      'API_URL': '',
      'GOOGLE_SERVER_CLIENT_ID': '',
      'GOOGLE_IOS_CLIENT_ID': '',
    });

    expect(missingIn(result), [
      'API_URL',
      'GOOGLE_SERVER_CLIENT_ID',
      'GOOGLE_IOS_CLIENT_ID',
    ]);
  });
}
