// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, unused_import, invalid_annotation_target, unnecessary_import

import 'package:json_annotation/json_annotation.dart';

part 'sign_in_dto.g.dart';

@JsonSerializable()
class SignInDto {
  const SignInDto({required this.email, required this.password});

  factory SignInDto.fromJson(Map<String, Object?> json) =>
      _$SignInDtoFromJson(json);

  final String email;
  final String password;

  Map<String, Object?> toJson() => _$SignInDtoToJson(this);
}
