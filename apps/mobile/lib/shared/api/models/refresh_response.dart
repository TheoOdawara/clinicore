// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, unused_import, invalid_annotation_target, unnecessary_import

import 'package:json_annotation/json_annotation.dart';

import 'session_tokens_response.dart';

part 'refresh_response.g.dart';

@JsonSerializable()
class RefreshResponse {
  const RefreshResponse({required this.tokens});

  factory RefreshResponse.fromJson(Map<String, Object?> json) =>
      _$RefreshResponseFromJson(json);

  final SessionTokensResponse tokens;

  Map<String, Object?> toJson() => _$RefreshResponseToJson(this);
}
