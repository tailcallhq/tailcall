package tailcall.runtime.internal

import zio.json.ast.Json
import zio.test.Gen

object JsonGen {
  val genJson: Gen[Any, Json] = Gen.suspend(Gen.oneOf(
    Gen.chunkOfBounded(0, 5)(for {
      key   <- Gen.string1(Gen.alphaChar)
      value <- genJson
    } yield (key, value)).map(Json.Obj(_)),
    Gen.chunkOfBounded(0, 5)(genJson).map(Json.Arr(_)),
    Gen.boolean.map(Json.Bool(_)),
    Gen.string.map(Json.Str(_)),
    Gen.double.map(Json.Num(_)),
    Gen.const(Json.Null),
  ))
}
