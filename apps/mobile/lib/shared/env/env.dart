class Configuration {
  const Configuration({
    required this.apiUrl,
    required this.googleServerClientId,
    required this.googleIosClientId,
  });

  final Uri apiUrl;
  final String googleServerClientId;
  final String googleIosClientId;
}

sealed class ConfigurationResult {
  const ConfigurationResult();
}

class ValidConfiguration extends ConfigurationResult {
  const ValidConfiguration(this.configuration);

  final Configuration configuration;
}

class MissingConfiguration extends ConfigurationResult {
  const MissingConfiguration(this.names);

  final List<String> names;
}

ConfigurationResult readConfiguration() {
  return parseConfiguration(const {
    'API_URL': String.fromEnvironment('API_URL'),
    'GOOGLE_SERVER_CLIENT_ID': String.fromEnvironment(
      'GOOGLE_SERVER_CLIENT_ID',
    ),
    'GOOGLE_IOS_CLIENT_ID': String.fromEnvironment('GOOGLE_IOS_CLIENT_ID'),
  });
}

ConfigurationResult parseConfiguration(Map<String, String> values) {
  Uri? absoluteUrlWithoutTrailingSlash(String? value) {
    if (value == null || value.endsWith('/')) {
      return null;
    }
    final url = Uri.tryParse(value);
    if (url == null || !url.isAbsolute || url.host.isEmpty) {
      return null;
    }
    return url;
  }

  final apiUrl = absoluteUrlWithoutTrailingSlash(values['API_URL']);
  final googleServerClientId = values['GOOGLE_SERVER_CLIENT_ID'] ?? '';
  final googleIosClientId = values['GOOGLE_IOS_CLIENT_ID'] ?? '';

  final missing = [
    if (apiUrl == null) 'API_URL',
    if (googleServerClientId.isEmpty) 'GOOGLE_SERVER_CLIENT_ID',
    if (googleIosClientId.isEmpty) 'GOOGLE_IOS_CLIENT_ID',
  ];
  if (apiUrl == null || missing.isNotEmpty) {
    return MissingConfiguration(missing);
  }

  return ValidConfiguration(
    Configuration(
      apiUrl: apiUrl,
      googleServerClientId: googleServerClientId,
      googleIosClientId: googleIosClientId,
    ),
  );
}
