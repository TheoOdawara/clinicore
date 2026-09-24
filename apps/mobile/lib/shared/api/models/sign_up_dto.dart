// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, unused_import, invalid_annotation_target, unnecessary_import

import 'package:json_annotation/json_annotation.dart';

part 'sign_up_dto.g.dart';

@JsonSerializable()
class SignUpDto {
  const SignUpDto({
    required this.password,
    required this.name,
    required this.email,
  });

  factory SignUpDto.fromJson(Map<String, Object?> json) =>
      _$SignUpDtoFromJson(json);

  /// At least 8 and at most 128 characters, with an uppercase letter, a digit and a character that is neither a letter nor a digit
  final String password;
  final String name;
  final String email;

  Map<String, Object?> toJson() => _$SignUpDtoToJson(this);
}
