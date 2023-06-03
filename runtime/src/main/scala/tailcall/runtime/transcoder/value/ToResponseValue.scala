package tailcall.runtime.transcoder.value

import caliban.ResponseValue
import tailcall.runtime.internal.{DynamicValueUtil, TValid}
import tailcall.runtime.transcoder.Transcoder
import zio.schema.DynamicValue

trait ToResponseValue {
  final def toResponseValue(input: DynamicValue): TValid[String, ResponseValue] = {
    input match {
      case DynamicValue.Sequence(values)               => TValid.foreach(values.toList)(toResponseValue)
          .map(ResponseValue.ListValue(_))
      case DynamicValue.Primitive(input, standardType) => Transcoder.toValue(input, standardType)
      case DynamicValue.Dictionary(chunks)             => TValid.foreachChunk(chunks) { case (k, v) =>
          DynamicValueUtil.toTyped[String](k) match {
            case Some(key) => toResponseValue(v).map(key -> _)
            case None      => TValid.fail("could not transform")
          }
        }.map(entries => ResponseValue.ObjectValue(entries.toList))
      case DynamicValue.Singleton(_)                   => TValid.fail("Can not transcode Singleton to ResponseValue")
      case DynamicValue.NoneValue                      => TValid.fail("Can not transcode NoneValue to ResponseValue")
      case DynamicValue.DynamicAst(_)                  => TValid.fail("Can not transcode DynamicAst to ResponseValue")
      case DynamicValue.SetValue(_)                    => TValid.fail("Can not transcode SetValue to ResponseValue")
      case DynamicValue.Record(_, fields)              => TValid.foreachIterable(fields) { case (k, v) =>
          toResponseValue(v).map(k -> _)
        }.map(entries => ResponseValue.ObjectValue(entries.toList))
      case DynamicValue.Enumeration(_, _)              => TValid.fail("Can not transcode Enumeration to ResponseValue")
      case DynamicValue.RightValue(_)                  => TValid.fail("Can not transcode RightValue to ResponseValue")
      case DynamicValue.SomeValue(input)               => toResponseValue(input)
      case DynamicValue.Tuple(_, _)                    => TValid.fail("Can not transcode Tuple to ResponseValue")
      case DynamicValue.LeftValue(_)                   => TValid.fail("Can not transcode LeftValue to ResponseValue")
      case DynamicValue.Error(_)                       => TValid.fail("Can not transcode Error to ResponseValue")
    }
  }
}
