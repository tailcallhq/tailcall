package tailcall.runtime.transcoder

import tailcall.runtime.transcoder.data._

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
    with Document2Blueprint
    with Document2Config
    with Orc2Blueprint
    with ToDynamicValue
    with ToInputValue
    with ToJsonAST
    with ToResponseValue
    with ToValue

object Transcoder extends Transcoder
