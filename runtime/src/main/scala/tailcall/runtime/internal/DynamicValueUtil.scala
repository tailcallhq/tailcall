package tailcall.runtime.internal

import caliban.InputValue.{ListValue => InputList, ObjectValue => InputObject, VariableValue}
import caliban.ResponseValue.{ListValue => ResponseList, ObjectValue => ResponseObject, StreamValue}
import caliban.Value.FloatValue.{BigDecimalNumber, DoubleNumber, FloatNumber}
import caliban.Value.IntValue.{BigIntNumber, IntNumber, LongNumber}
import caliban.Value.{BooleanValue, EnumValue, NullValue, StringValue}
import caliban.{InputValue, ResponseValue, Value}
import zio.Chunk
import zio.json.ast.Json
import zio.schema.{DynamicValue, Schema, StandardType, TypeId}

import scala.collection.immutable.ListMap

object DynamicValueUtil {
  def asString(dv: DynamicValue): Option[String] =
    dv match {
      case DynamicValue.Primitive(value, _) => Some(value.toString)
      case _                                => None
    }

  private def toValue[A](value: A, standardType: StandardType[A]): Value =
    standardType match {
      case StandardType.StringType         => Value.StringValue(value.toString)
      case StandardType.IntType            => Value.IntValue(value.toString.toInt)
      case StandardType.MonthDayType       => Value.StringValue(value.toString)
      case StandardType.LocalDateTimeType  => Value.StringValue(value.toString)
      case StandardType.BoolType           => Value.BooleanValue(value.toString.toBoolean)
      case StandardType.LocalTimeType      => Value.StringValue(value.toString)
      case StandardType.OffsetDateTimeType => Value.StringValue(value.toString)
      case StandardType.MonthType          => Value.StringValue(value.toString)
      case StandardType.ShortType          => Value.IntValue(value.toString.toShort)
      case StandardType.ZoneIdType         => Value.StringValue(value.toString)
      case StandardType.BigDecimalType     => Value.FloatValue(BigDecimal(value.toString))
      case StandardType.YearType           => Value.IntValue(value.toString.toInt)
      case StandardType.ByteType           => Value.IntValue(value.toString.toByte)
      case StandardType.UUIDType           => Value.StringValue(value.toString)
      case StandardType.PeriodType         => Value.StringValue(value.toString)
      case StandardType.LongType           => Value.IntValue(value.toString.toLong)
      case StandardType.ZoneOffsetType     => Value.StringValue(value.toString)
      case StandardType.BigIntegerType     => Value.IntValue(BigInt(value.toString))
      case StandardType.OffsetTimeType     => Value.StringValue(value.toString)
      case StandardType.UnitType           => Value.NullValue
      case StandardType.DoubleType         => Value.FloatValue(value.toString.toDouble)
      case StandardType.InstantType        => Value.StringValue(value.toString)
      case StandardType.FloatType          => Value.FloatValue(value.toString.toFloat)
      case StandardType.LocalDateType      => Value.StringValue(value.toString)
      case StandardType.ZonedDateTimeType  => Value.StringValue(value.toString)
      case StandardType.YearMonthType      => Value.StringValue(value.toString)
      case StandardType.CharType           => Value.StringValue(value.toString)
      case StandardType.BinaryType         => Value
          .StringValue(java.util.Base64.getEncoder.encodeToString(value.asInstanceOf[Chunk[Byte]].toArray))
      case StandardType.DurationType       => Value.StringValue(value.toString)
      case StandardType.DayOfWeekType      => Value.StringValue(value.toString)
    }

  def toResponseValue(input: DynamicValue): ResponseValue = {
    input match {
      case DynamicValue.Sequence(values)               => ResponseValue.ListValue(values.map(toResponseValue).toList)
      case DynamicValue.Primitive(value, standardType) => toValue(value, standardType)
      case DynamicValue.Dictionary(chunks)             => ResponseValue.ObjectValue(chunks.map { case (k, v) =>
          toTyped[String](k).getOrElse(throw new Error("could not transform")) -> toResponseValue(v)
        }.toList)
      case DynamicValue.Singleton(_)                   => Value.NullValue
      case DynamicValue.NoneValue                      => Value.NullValue
      case DynamicValue.DynamicAst(_)                  => Value.NullValue
      case DynamicValue.SetValue(_)                    => Value.NullValue
      case DynamicValue.Record(_, fields)              => ResponseValue.ObjectValue(fields.map { case (k, v) =>
          k -> toResponseValue(v)
        }.toList)
      case DynamicValue.Enumeration(_, _)              => Value.NullValue
      case DynamicValue.RightValue(_)                  => Value.NullValue
      case DynamicValue.SomeValue(input)               => toResponseValue(input)
      case DynamicValue.Tuple(_, _)                    => Value.NullValue
      case DynamicValue.LeftValue(_)                   => Value.NullValue
      case DynamicValue.Error(_)                       => Value.NullValue
    }
  }

  def toInputValue(input: DynamicValue): InputValue = {
    input match {
      case DynamicValue.Sequence(values)               => InputValue.ListValue(values.map(toInputValue).toList)
      case DynamicValue.Primitive(value, standardType) => toValue(value, standardType)
      case DynamicValue.Dictionary(chunks)             => InputValue.ObjectValue(chunks.map { case (k, v) =>
          toTyped[String](k).getOrElse(throw new Error("could not transform")) -> toInputValue(v)
        }.toMap)
      case DynamicValue.Singleton(_)                   => Value.NullValue
      case DynamicValue.NoneValue                      => Value.NullValue
      case DynamicValue.DynamicAst(_)                  => Value.NullValue
      case DynamicValue.SetValue(_)                    => Value.NullValue
      case DynamicValue.Record(_, b)      => InputValue.ObjectValue(b.map { case (k, v) => k -> toInputValue(v) })
      case DynamicValue.Enumeration(_, _) => Value.NullValue
      case DynamicValue.RightValue(_)     => Value.NullValue
      case DynamicValue.SomeValue(input)  => toInputValue(input)
      case DynamicValue.Tuple(_, _)       => Value.NullValue
      case DynamicValue.LeftValue(_)      => Value.NullValue
      case DynamicValue.Error(_)          => Value.NullValue
    }
  }

  def toTyped[A](d: DynamicValue)(implicit schema: Schema[A]): Option[A] = d.toTypedValueOption(schema)

  def getPath(d: DynamicValue, path: List[String]): Option[DynamicValue] =
    path match {
      case Nil          => Some(d)
      case head :: tail => d match {
          case DynamicValue.Record(_, b)  => b.get(head).flatMap(getPath(_, tail))
          case DynamicValue.SomeValue(a)  => getPath(a, path)
          case DynamicValue.Sequence(a)   => head.toIntOption.flatMap(a.lift).flatMap(getPath(_, tail))
          case DynamicValue.Dictionary(b) =>
            val stringTag = StandardType.StringType.asInstanceOf[StandardType[Any]]
            b.collect { case (DynamicValue.Primitive(`head`, `stringTag`), value) => value }.headOption
              .flatMap(getPath(_, tail))
          case _                          => None
        }
    }

  def fromResponseValue(input: ResponseValue): DynamicValue = {
    input match {
      case ResponseList(values)    => DynamicValue(values.map(fromResponseValue(_)))
      case ResponseObject(fields)  => DynamicValue(fields.toMap.map { case (k, v) => k -> fromResponseValue(v) })
      case StringValue(value)      => DynamicValue(value)
      case NullValue               => DynamicValue.NoneValue
      case BooleanValue(value)     => DynamicValue(value)
      case BigDecimalNumber(value) => DynamicValue(value)
      case DoubleNumber(value)     => DynamicValue(value)
      case FloatNumber(value)      => DynamicValue(value)
      case BigIntNumber(value)     => DynamicValue(value)
      case IntNumber(value)        => DynamicValue(value)
      case LongNumber(value)       => DynamicValue(value)
      case EnumValue(_)            => DynamicValue.NoneValue
      case StreamValue(_)          => DynamicValue.NoneValue
    }
  }

  def fromInputValue(input: InputValue): DynamicValue = {
    input match {
      case InputList(values)       => DynamicValue(values.map(fromInputValue(_)))
      case InputObject(fields)     => DynamicValue(fields.map { case (k, v) => k -> fromInputValue(v) })
      case StringValue(value)      => DynamicValue(value)
      case NullValue               => DynamicValue.NoneValue
      case BooleanValue(value)     => DynamicValue(value)
      case BigDecimalNumber(value) => DynamicValue(value)
      case DoubleNumber(value)     => DynamicValue(value)
      case FloatNumber(value)      => DynamicValue(value)
      case BigIntNumber(value)     => DynamicValue(value)
      case IntNumber(value)        => DynamicValue(value)
      case LongNumber(value)       => DynamicValue(value)
      case EnumValue(_)            => DynamicValue.NoneValue
      case VariableValue(_)        => DynamicValue.NoneValue
    }
  }

  def record(fields: (String, DynamicValue)*): DynamicValue =
    DynamicValue.Record(TypeId.Structural, ListMap.from(fields))

  def fromJson(json: Json): DynamicValue =
    json match {
      case Json.Obj(fields)   => DynamicValue
          .Record(TypeId.Structural, ListMap.from(fields.map { case (k, v) => k -> fromJson(v) }))
      case Json.Arr(elements) => DynamicValue(elements.map(fromJson))
      case Json.Bool(value)   => DynamicValue(value)
      case Json.Str(value)    => DynamicValue(value)
      case Json.Num(value)    => DynamicValue(value)
      case Json.Null          => DynamicValue.NoneValue
    }
}
