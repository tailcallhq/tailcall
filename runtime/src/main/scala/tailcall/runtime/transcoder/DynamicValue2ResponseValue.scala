package tailcall.runtime.transcoder

import caliban.{ResponseValue, Value}
import tailcall.runtime.internal.DynamicValueUtil
import tailcall.runtime.transcoder.Transcoder.{Syntax, TExit}
import zio.schema.DynamicValue

object DynamicValue2ResponseValue {

  def toResponseValue(input: DynamicValue): TExit[String, ResponseValue] = {
    input match {
      case DynamicValue.Sequence(values) => TExit.foreach(values.toList)(toResponseValue).map(ResponseValue.ListValue)
      case input @ DynamicValue.Primitive(_, _) => TExit.succeed(input.transcode[Value])
      case DynamicValue.Dictionary(chunks)      => TExit.foreachChunk(chunks) { case (k, v) =>
          DynamicValueUtil.toTyped[String](k) match {
            case Some(key) => toResponseValue(v).map(key -> _)
            case None      => TExit.fail("could not transform")
          }
        }.map(entries => ResponseValue.ObjectValue(entries.toList))
      case DynamicValue.Singleton(_)            => TExit.fail("Can not transcode Singleton to ResponseValue")
      case DynamicValue.NoneValue               => TExit.fail("Can not transcode NoneValue to ResponseValue")
      case DynamicValue.DynamicAst(_)           => TExit.fail("Can not transcode DynamicAst to ResponseValue")
      case DynamicValue.SetValue(_)             => TExit.fail("Can not transcode SetValue to ResponseValue")
      case DynamicValue.Record(_, fields)       => TExit.foreachIterable(fields) { case (k, v) =>
          toResponseValue(v).map(k -> _)
        }.map(entries => ResponseValue.ObjectValue(entries.toList))
      case DynamicValue.Enumeration(_, _)       => TExit.fail("Can not transcode Enumeration to ResponseValue")
      case DynamicValue.RightValue(_)           => TExit.fail("Can not transcode RightValue to ResponseValue")
      case DynamicValue.SomeValue(input)        => toResponseValue(input)
      case DynamicValue.Tuple(_, _)             => TExit.fail("Can not transcode Tuple to ResponseValue")
      case DynamicValue.LeftValue(_)            => TExit.fail("Can not transcode LeftValue to ResponseValue")
      case DynamicValue.Error(_)                => TExit.fail("Can not transcode Error to ResponseValue")
    }
  }
}
