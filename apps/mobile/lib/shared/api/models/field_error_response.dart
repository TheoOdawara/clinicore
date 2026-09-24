// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, unused_import, invalid_annotation_target, unnecessary_import

import 'package:json_annotation/json_annotation.dart';

part 'field_error_response.g.dart';

@JsonSerializable()
class FieldErrorResponse {
  const FieldErrorResponse({required this.pointer, required this.code});

  factory FieldErrorResponse.fromJson(Map<String, Object?> json) =>
      _$FieldErrorResponseFromJson(json);

  final String pointer;
  final String code;

  Map<String, Object?> toJson() => _$FieldErrorResponseToJson(this);
}
