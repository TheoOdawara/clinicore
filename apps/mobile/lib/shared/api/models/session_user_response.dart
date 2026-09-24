// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, unused_import, invalid_annotation_target, unnecessary_import

import 'package:json_annotation/json_annotation.dart';

part 'session_user_response.g.dart';

@JsonSerializable()
class SessionUserResponse {
  const SessionUserResponse({
    required this.id,
    required this.name,
    required this.email,
    required this.emailVerified,
    required this.image,
  });

  factory SessionUserResponse.fromJson(Map<String, Object?> json) =>
      _$SessionUserResponseFromJson(json);

  final String id;
  final String name;
  final String email;
  final bool emailVerified;
  final String? image;

  Map<String, Object?> toJson() => _$SessionUserResponseToJson(this);
}
