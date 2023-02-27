package tailcall.gateway

import tailcall.gateway.remote.operations._

package object remote extends MathOps with DynamicValueOps with BooleanOps with MapOps with OptionOps {}
