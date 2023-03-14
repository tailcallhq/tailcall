package tailcall.runtime.internal

import caliban.{InputValue, ResponseValue, Value}
import zio.Chunk
import zio.json.ast.Json
import zio.schema.{DynamicValue, Schema, StandardType, TypeId}

import java.math.BigDecimal
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
      case StandardType.BigDecimalType     => Value.StringValue(value.toString)
      case StandardType.YearType           => Value.IntValue(value.toString.toInt)
      case StandardType.ByteType           => Value.IntValue(value.toString.toByte)
      case StandardType.UUIDType           => Value.StringValue(value.toString)
      case StandardType.PeriodType         => Value.StringValue(value.toString)
      case StandardType.LongType           => Value.StringValue(value.toString)
      case StandardType.ZoneOffsetType     => Value.StringValue(value.toString)
      case StandardType.BigIntegerType     => Value.StringValue(value.toString)
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
      case DynamicValue.Singleton(_)                   => ???
      case DynamicValue.NoneValue                      => Value.NullValue
      case DynamicValue.DynamicAst(_)                  => ???
      case DynamicValue.SetValue(_)                    => ???
      case DynamicValue.Record(_, fields)              => ResponseValue.ObjectValue(fields.map { case (k, v) =>
          k -> toResponseValue(v)
        }.toList)
      case DynamicValue.Enumeration(_, _)              => ???
      case DynamicValue.RightValue(_)                  => ???
      case DynamicValue.SomeValue(input)               => toResponseValue(input)
      case DynamicValue.Tuple(_, _)                    => ???
      case DynamicValue.LeftValue(_)                   => ???
      case DynamicValue.Error(_)                       => ???
    }
  }

  def toInputValue(input: DynamicValue): InputValue = {
    input match {
      case DynamicValue.Sequence(values)               => InputValue.ListValue(values.map(toInputValue).toList)
      case DynamicValue.Primitive(value, standardType) => toValue(value, standardType)
      case DynamicValue.Dictionary(chunks)             => InputValue.ObjectValue(chunks.map { case (k, v) =>
          toTyped[String](k).getOrElse(throw new Error("could not transform")) -> toInputValue(v)
        }.toMap)
      case DynamicValue.Singleton(_)                   => ???
      case DynamicValue.NoneValue                      => ???
      case DynamicValue.DynamicAst(_)                  => ???
      case DynamicValue.SetValue(_)                    => ???
      case DynamicValue.Record(_, b)      => InputValue.ObjectValue(b.map { case (k, v) => k -> toInputValue(v) })
      case DynamicValue.Enumeration(_, _) => ???
      case DynamicValue.RightValue(_)     => ???
      case DynamicValue.SomeValue(_)      => ???
      case DynamicValue.Tuple(_, _)       => ???
      case DynamicValue.LeftValue(_)      => ???
      case DynamicValue.Error(_)          => ???
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

  // TODO: clean up
  def fromInputValue(input: InputValue): DynamicValue = {
    import caliban.InputValue.{ListValue, ObjectValue, VariableValue}
    import caliban.Value.FloatValue.{BigDecimalNumber, DoubleNumber, FloatNumber}
    import caliban.Value.IntValue.{BigIntNumber, IntNumber, LongNumber}
    import caliban.Value.{BooleanValue, EnumValue, NullValue, StringValue}

    input match {
      case ListValue(values)       => DynamicValue(values.map(fromInputValue(_)))
      case ObjectValue(fields)     => DynamicValue(fields.map { case (k, v) => k -> fromInputValue(v) })
      case StringValue(value)      => DynamicValue(value)
      case NullValue               => ???
      case BooleanValue(value)     => DynamicValue(value)
      case BigDecimalNumber(value) => DynamicValue(value)
      case DoubleNumber(value)     => DynamicValue(value)
      case FloatNumber(value)      => DynamicValue(value)
      case BigIntNumber(value)     => DynamicValue(value)
      case IntNumber(value)        => DynamicValue(value)
      case LongNumber(value)       => DynamicValue(value)
      case EnumValue(_)            => ???
      case VariableValue(_)        => ???
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

  private def toJsonPrimitive[A](value: A, standardType: StandardType[A]): Json =
    standardType match {
      case StandardType.UnitType           => Json.Str(value.toString)
      case StandardType.StringType         => Json.Str(value.toString)
      case StandardType.BoolType           => Json.Bool(value.asInstanceOf[Boolean])
      case StandardType.ByteType           => Json.Str(value.toString)
      case StandardType.ShortType          => Json.Str(value.toString)
      case StandardType.IntType            => Json.Num(value.asInstanceOf[Int])
      case StandardType.LongType           => Json.Num(value.asInstanceOf[Long])
      case StandardType.FloatType          => Json.Num(value.asInstanceOf[Float])
      case StandardType.DoubleType         => Json.Num(value.asInstanceOf[Double])
      case StandardType.BinaryType         => Json
          .Str(java.util.Base64.getEncoder.encodeToString(value.asInstanceOf[Chunk[Byte]].toArray))
      case StandardType.CharType           => Json.Str(value.toString)
      case StandardType.UUIDType           => Json.Str(value.toString)
      case StandardType.BigDecimalType     => Json.Num(value.asInstanceOf[BigDecimal])
      case StandardType.BigIntegerType     => Json.Str(value.toString)
      case StandardType.DayOfWeekType      => Json.Str(value.toString)
      case StandardType.MonthType          => Json.Str(value.toString)
      case StandardType.MonthDayType       => Json.Str(value.toString)
      case StandardType.PeriodType         => Json.Str(value.toString)
      case StandardType.YearType           => Json.Str(value.toString)
      case StandardType.YearMonthType      => Json.Str(value.toString)
      case StandardType.ZoneIdType         => Json.Str(value.toString)
      case StandardType.ZoneOffsetType     => Json.Str(value.toString)
      case StandardType.DurationType       => Json.Str(value.toString)
      case StandardType.InstantType        => Json.Str(value.toString)
      case StandardType.LocalDateType      => Json.Str(value.toString)
      case StandardType.LocalTimeType      => Json.Str(value.toString)
      case StandardType.LocalDateTimeType  => Json.Str(value.toString)
      case StandardType.OffsetTimeType     => Json.Str(value.toString)
      case StandardType.OffsetDateTimeType => Json.Str(value.toString)
      case StandardType.ZonedDateTimeType  => Json.Str(value.toString)
    }

  def toJson(d: DynamicValue): Json =
    d match {
      case DynamicValue.Record(_, values) => Json.Obj(Chunk.from(values.map { case (k, v) => k -> toJson(v) }))
      case DynamicValue.Enumeration(_, (name, value))  => Json.Obj(Chunk(name -> toJson(value)))
      case DynamicValue.Sequence(values)               => Json.Arr(Chunk.from(values.map(toJson)))
      case DynamicValue.Dictionary(_)                  => ???
      case DynamicValue.SetValue(values)               => Json.Arr(Chunk.from(values.map(toJson)))
      case DynamicValue.Primitive(value, standardType) => toJsonPrimitive(value, standardType)
      case DynamicValue.Singleton(_)                   => ???
      case DynamicValue.SomeValue(value)               => toJson(value)
      case DynamicValue.NoneValue                      => Json.Null
      case DynamicValue.Tuple(left, right)             => Json.Arr(Chunk(toJson(left), toJson(right)))
      case DynamicValue.LeftValue(value)               => toJson(value)
      case DynamicValue.RightValue(value)              => toJson(value)
      case DynamicValue.DynamicAst(_)                  => ???
      case DynamicValue.Error(_)                       => ???
    }
}
