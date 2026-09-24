// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, unused_import, invalid_annotation_target, unnecessary_import

import 'package:json_annotation/json_annotation.dart';

part 'password_reset_confirmation_dto.g.dart';

@JsonSerializable()
class PasswordResetConfirmationDto {
  const PasswordResetConfirmationDto({
    required this.token,
    required this.newPassword,
  });

  factory PasswordResetConfirmationDto.fromJson(Map<String, Object?> json) =>
      _$PasswordResetConfirmationDtoFromJson(json);

  /// The token from the emailed link: 43 base64url characters
  final String token;

  /// At least 8 and at most 128 characters, with an uppercase letter, a digit and a character that is neither a letter nor a digit
  final String newPassword;

  Map<String, Object?> toJson() => _$PasswordResetConfirmationDtoToJson(this);
}
