package tailcall.runtime

import tailcall.runtime.remote.operations._

package object remote extends MathOps with DynamicValueOps with BooleanOps with MapOps with OptionOps {}
