// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'get_health_response.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

GetHealthResponse _$GetHealthResponseFromJson(Map<String, dynamic> json) =>
    GetHealthResponse(
      status: json['status'] as String?,
      info: (json['info'] as Map<String, dynamic>?)?.map(
        (k, e) => MapEntry(k, Info.fromJson(e as Map<String, dynamic>)),
      ),
      error: (json['error'] as Map<String, dynamic>?)?.map(
        (k, e) => MapEntry(k, Error.fromJson(e as Map<String, dynamic>)),
      ),
      details: (json['details'] as Map<String, dynamic>?)?.map(
        (k, e) => MapEntry(k, Details.fromJson(e as Map<String, dynamic>)),
      ),
    );

Map<String, dynamic> _$GetHealthResponseToJson(GetHealthResponse instance) =>
    <String, dynamic>{
      'status': instance.status,
      'info': instance.info,
      'error': instance.error,
      'details': instance.details,
    };
