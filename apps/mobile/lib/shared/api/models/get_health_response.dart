// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, unused_import, invalid_annotation_target, unnecessary_import

import 'package:json_annotation/json_annotation.dart';

import 'info.dart';
import 'error.dart';
import 'details.dart';

part 'get_health_response.g.dart';

@JsonSerializable()
class GetHealthResponse {
  const GetHealthResponse({this.status, this.info, this.error, this.details});

  factory GetHealthResponse.fromJson(Map<String, Object?> json) =>
      _$GetHealthResponseFromJson(json);

  final String? status;
  final Map<String, Info>? info;
  final Map<String, Error>? error;
  final Map<String, Details>? details;

  Map<String, Object?> toJson() => _$GetHealthResponseToJson(this);
}
