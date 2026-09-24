// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, unused_import, invalid_annotation_target, unnecessary_import

import 'package:json_annotation/json_annotation.dart';

import 'field_error_response.dart';

part 'problem_response.g.dart';

@JsonSerializable()
class ProblemResponse {
  const ProblemResponse({
    required this.type,
    required this.title,
    required this.status,
    this.errors,
  });

  factory ProblemResponse.fromJson(Map<String, Object?> json) =>
      _$ProblemResponseFromJson(json);

  final String type;
  final String title;
  final num status;
  final List<FieldErrorResponse>? errors;

  Map<String, Object?> toJson() => _$ProblemResponseToJson(this);
}
