// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, unused_import, invalid_annotation_target, unnecessary_import

import 'package:json_annotation/json_annotation.dart';

part 'token_dto.g.dart';

@JsonSerializable()
class TokenDto {
  const TokenDto({required this.token});

  factory TokenDto.fromJson(Map<String, Object?> json) =>
      _$TokenDtoFromJson(json);

  /// The token from the emailed link: 43 base64url characters
  final String token;

  Map<String, Object?> toJson() => _$TokenDtoToJson(this);
}
