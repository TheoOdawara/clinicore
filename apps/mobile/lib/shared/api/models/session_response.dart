// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, unused_import, invalid_annotation_target, unnecessary_import

import 'package:json_annotation/json_annotation.dart';

import 'session_tokens_response.dart';
import 'session_user_response.dart';

part 'session_response.g.dart';

@JsonSerializable()
class SessionResponse {
  const SessionResponse({required this.user, this.tokens});

  factory SessionResponse.fromJson(Map<String, Object?> json) =>
      _$SessionResponseFromJson(json);

  final SessionUserResponse user;

  /// Only with Clinicore-Client: mobile, which gets no cookie
  final SessionTokensResponse? tokens;

  Map<String, Object?> toJson() => _$SessionResponseToJson(this);
}
