// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'password_reset_confirmation_dto.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

PasswordResetConfirmationDto _$PasswordResetConfirmationDtoFromJson(
  Map<String, dynamic> json,
) => PasswordResetConfirmationDto(
  token: json['token'] as String,
  newPassword: json['newPassword'] as String,
);

Map<String, dynamic> _$PasswordResetConfirmationDtoToJson(
  PasswordResetConfirmationDto instance,
) => <String, dynamic>{
  'token': instance.token,
  'newPassword': instance.newPassword,
};
