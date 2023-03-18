package tailcall.runtime.transcoder

import caliban.{ResponseValue, Value}
import tailcall.runtime.internal.DynamicValueUtil.toTyped
import zio.schema.DynamicValue

object DynamicValue2ResponseValue {

  def toResponseValue(input: DynamicValue): ResponseValue = {
    input match {
      case DynamicValue.Sequence(values)        => ResponseValue.ListValue(values.map(toResponseValue).toList)
      case input @ DynamicValue.Primitive(_, _) => input.transcode[Value]
      case DynamicValue.Dictionary(chunks)      => ResponseValue.ObjectValue(chunks.map { case (k, v) =>
          // TODO: use TExit.fail
          toTyped[String](k).getOrElse(throw new Error("could not transform")) -> toResponseValue(v)
        }.toList)
      case DynamicValue.Singleton(_)            => Value.NullValue
      case DynamicValue.NoneValue               => Value.NullValue
      case DynamicValue.DynamicAst(_)           => Value.NullValue
      case DynamicValue.SetValue(_)             => Value.NullValue
      case DynamicValue.Record(_, fields)       => ResponseValue.ObjectValue(fields.map { case (k, v) =>
          k -> toResponseValue(v)
        }.toList)
      case DynamicValue.Enumeration(_, _)       => Value.NullValue
      case DynamicValue.RightValue(_)           => Value.NullValue
      case DynamicValue.SomeValue(input)        => toResponseValue(input)
      case DynamicValue.Tuple(_, _)             => Value.NullValue
      case DynamicValue.LeftValue(_)            => Value.NullValue
      case DynamicValue.Error(_)                => Value.NullValue
    }
  }
}
