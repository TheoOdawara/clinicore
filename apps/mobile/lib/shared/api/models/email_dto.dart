// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, unused_import, invalid_annotation_target, unnecessary_import

import 'package:json_annotation/json_annotation.dart';

part 'email_dto.g.dart';

@JsonSerializable()
class EmailDto {
  const EmailDto({required this.email});

  factory EmailDto.fromJson(Map<String, Object?> json) =>
      _$EmailDtoFromJson(json);

  final String email;

  Map<String, Object?> toJson() => _$EmailDtoToJson(this);
}
