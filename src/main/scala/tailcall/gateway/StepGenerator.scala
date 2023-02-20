package tailcall.gateway

import caliban.schema.Step
import caliban.{ResponseValue, Value}
import tailcall.gateway.StepGenerator.RemoteStep
import tailcall.gateway.ast.{Context, Orc}
import tailcall.gateway.lambda.LambdaRuntime
import tailcall.gateway.remote.Remote
import zio.query.ZQuery
import zio.schema.{DynamicValue, StandardType}

final class StepGenerator(orc: Orc) {
  def toValue(value: Any, standardType: StandardType[_]): Value =
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
          .StringValue(java.util.Base64.getEncoder.encodeToString(value.asInstanceOf[Array[Byte]]))
      case StandardType.DurationType       => Value.StringValue(value.toString)
      case StandardType.DayOfWeekType      => Value.StringValue(value.toString)
    }
  def toValue(input: DynamicValue): ResponseValue               = {
    input match {
      case DynamicValue.Sequence(values)               => ResponseValue.ListValue(values.map(toValue).toList)
      case DynamicValue.Primitive(value, standardType) => toValue(value, standardType)
      case DynamicValue.Dictionary(_)                  => ???
      case DynamicValue.Singleton(_)                   => ???
      case DynamicValue.NoneValue                      => ???
      case DynamicValue.DynamicAst(_)                  => ???
      case DynamicValue.SetValue(_)                    => ???
      case DynamicValue.Record(_, _)                   => ???
      case DynamicValue.Enumeration(_, _)              => ???
      case DynamicValue.RightValue(_)                  => ???
      case DynamicValue.SomeValue(_)                   => ???
      case DynamicValue.Tuple(_, _)                    => ???
      case DynamicValue.LeftValue(_)                   => ???
      case DynamicValue.Error(_)                       => ???
    }
  }

  def gen(orc: Orc, context: Remote[Context]): RemoteStep = {
    orc match {
      case Orc.OrcValue(dynamicValue)  => Step.PureStep(toValue(dynamicValue))
      case Orc.OrcObject(name, fields) => Step
          .ObjectStep(name, fields.map { case (name, orc) => (name, gen(orc, context)) })
      case Orc.OrcList(values)         => Step.ListStep(values.map(gen(_, context)))
      case Orc.OrcFunction(fun)        => Step.QueryStep(ZQuery.fromZIO(fun(context).evaluate.map(gen(_, context))))
    }
  }

  def gen: RemoteStep = { gen(orc, Remote(Context(DynamicValue(())))) }
}

object StepGenerator {
  type RemoteStep = Step[LambdaRuntime]

  sealed trait GraphQLSchemaGeneratorError extends Throwable
  case object QueryNotFound                extends GraphQLSchemaGeneratorError
}
