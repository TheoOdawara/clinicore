// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, unused_import, invalid_annotation_target, unnecessary_import

import 'package:json_annotation/json_annotation.dart';

part 'session_tokens_response.g.dart';

@JsonSerializable()
class SessionTokensResponse {
  const SessionTokensResponse({
    required this.accessToken,
    required this.refreshToken,
    required this.accessTokenExpiresIn,
  });

  factory SessionTokensResponse.fromJson(Map<String, Object?> json) =>
      _$SessionTokensResponseFromJson(json);

  /// Sent as Authorization: Bearer
  final String accessToken;

  /// <session uuid>.<43 base64url characters>
  final String refreshToken;
  final num accessTokenExpiresIn;

  Map<String, Object?> toJson() => _$SessionTokensResponseToJson(this);
}
