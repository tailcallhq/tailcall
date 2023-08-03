package tailcall.runtime.transcoder

import tailcall.runtime.internal.TValid
import tailcall.runtime.model.{Blueprint, Config, Endpoint}
import tailcall.runtime.transcoder.Endpoint2Config.NameGenerator
import tailcall.runtime.transcoder.value._

/**
 * A transcoder is a function that takes an A and returns a
 * B, or an error. It can be composed using the >>> operator
 * with other transcoders to create a pipeline. A transcoder
 * between A ~> C can be derived provided there exists a B
 * such that a transcoder from A ~> B exists and a
 * transcoder from B ~> C already exists.
 */
sealed trait Transcoder
    extends Blueprint2Document
    with Config2Blueprint
    with Config2Document
    with Document2Blueprint
    with Document2Config
    with Document2SDL
    with Endpoint2Config
    with JsonValue2TSchema
    with SDL2JsonLines
    with ToDynamicValue
    with ToInputValue
    with ToJsonAST
    with ToResponseValue
    with ToValue

object Transcoder extends Transcoder {
  def toBlueprint(endpoint: Endpoint, nameGen: NameGenerator): TValid[String, Blueprint] =
    toConfig(endpoint, nameGen).flatMap(toBlueprint(_))

  def toSDL(endpoint: Endpoint, nameGenerator: NameGenerator): TValid[String, String] =
    toConfig(endpoint, nameGenerator).flatMap(config => toSDL(config.compress, true))

  def toSDL(config: Config, asConfig: Boolean): TValid[String, String] = {
    if (asConfig) toDocument(config).flatMap(toSDL) else toBlueprint(config).flatMap(toDocument).flatMap(toSDL)
  }
}
