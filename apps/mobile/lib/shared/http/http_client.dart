import 'package:dio/dio.dart';

Dio createHttpClient(Uri apiUrl) {
  return Dio(
    BaseOptions(
      baseUrl: apiUrl.toString(),
      headers: {'Clinicore-Client': 'mobile'},
    ),
  );
}
