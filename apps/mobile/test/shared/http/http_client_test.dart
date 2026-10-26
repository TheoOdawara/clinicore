import 'dart:typed_data';

import 'package:clinicore/shared/http/http_client.dart';
import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';

class RecordingAdapter implements HttpClientAdapter {
  RequestOptions? lastRequest;

  @override
  Future<ResponseBody> fetch(
    RequestOptions options,
    Stream<Uint8List>? requestStream,
    Future<void>? cancelFuture,
  ) async {
    lastRequest = options;
    return ResponseBody.fromString('{}', 200);
  }

  @override
  void close({bool force = false}) {}
}

void main() {
  test('sends every request to API_URL as the mobile client', () async {
    final adapter = RecordingAdapter();
    final client = createHttpClient(Uri.parse('https://api.clinicore.com.br'))
      ..httpClientAdapter = adapter;

    await client.get<void>('/sessions/current');

    final request = adapter.lastRequest!;
    expect(
      request.uri,
      Uri.parse('https://api.clinicore.com.br/sessions/current'),
    );
    expect(request.headers['Clinicore-Client'], 'mobile');
  });
}
