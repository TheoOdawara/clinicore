// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'problem_response.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

ProblemResponse _$ProblemResponseFromJson(Map<String, dynamic> json) =>
    ProblemResponse(
      type: json['type'] as String,
      title: json['title'] as String,
      status: json['status'] as num,
      errors: (json['errors'] as List<dynamic>?)
          ?.map((e) => FieldErrorResponse.fromJson(e as Map<String, dynamic>))
          .toList(),
    );

Map<String, dynamic> _$ProblemResponseToJson(ProblemResponse instance) =>
    <String, dynamic>{
      'type': instance.type,
      'title': instance.title,
      'status': instance.status,
      'errors': instance.errors,
    };
