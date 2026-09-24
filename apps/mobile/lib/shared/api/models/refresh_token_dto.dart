// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, unused_import, invalid_annotation_target, unnecessary_import

import 'package:json_annotation/json_annotation.dart';

part 'refresh_token_dto.g.dart';

@JsonSerializable()
class RefreshTokenDto {
  const RefreshTokenDto({this.refreshToken});

  factory RefreshTokenDto.fromJson(Map<String, Object?> json) =>
      _$RefreshTokenDtoFromJson(json);

  /// Required with Clinicore-Client: mobile, ignored without it: <session uuid>.<43 base64url characters>
  final String? refreshToken;

  Map<String, Object?> toJson() => _$RefreshTokenDtoToJson(this);
}
