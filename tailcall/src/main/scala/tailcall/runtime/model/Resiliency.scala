package tailcall.runtime.model

import zio.Duration

sealed trait Resiliency
object Resiliency {

  final case class CircuitBreaker(threshold: Threshold, reset: Reset) extends Resiliency

  sealed trait Threshold {
    self =>
    final def &(other: Threshold): Threshold = Threshold.and(self, other)
    final def |(other: Threshold): Threshold = Threshold.or(self, other)
  }

  object Threshold {
    final case class Count(value: Int)                                                      extends Threshold
    final case class Percentage(value: Int)                                                 extends Threshold
    final case class Combine(self: Threshold, other: Threshold, operator: Combine.Operator) extends Threshold
    object Combine {
      sealed trait Operator
      object Operator {
        case object And extends Operator
        case object Or  extends Operator
      }
    }

    def count(value: Int): Threshold                      = Count(value)
    def percentage(value: Int): Threshold                 = Percentage(value)
    def and(self: Threshold, other: Threshold): Threshold = Combine(self, other, Combine.Operator.And)
    def or(self: Threshold, other: Threshold): Threshold  = Combine(self, other, Combine.Operator.Or)
  }

  sealed trait Reset
  object Reset {
    final case class Fixed(duration: Duration)                                              extends Reset
    final case class Exponential(minDuration: Duration, maxDuration: Duration, factor: Int) extends Reset
  }

  sealed trait RetryPolicy
  object RetryPolicy {
    case object Immediate                                                   extends RetryPolicy
    final case class Fixed(duration: Duration, max: Int)                    extends RetryPolicy
    final case class Exponential(duration: Duration, max: Int, factor: Int) extends RetryPolicy
  }

  final case class RateLimiter(duration: Duration, count: Long)   extends Resiliency
  final case class Retry(policy: RetryPolicy)                     extends Resiliency
  final case class BulkHead(maxConcurrency: Long, maxQueue: Long) extends Resiliency
  final case class Timeout(duration: Duration)                    extends Resiliency

  // TODO: re-think about the schedule and it's integration with Resiliency
  sealed trait Schedule {
    self =>
    final def &(other: Schedule): Schedule = Schedule.and(self, other)
    final def |(other: Schedule): Schedule = Schedule.or(self, other)
  }

  object Schedule {
    final case class Fixed(duration: Duration)                                              extends Schedule
    final case class Exponential(minDuration: Duration, maxDuration: Duration, factor: Int) extends Schedule
    case object Immediate                                                                   extends Schedule
    case class Jittered(schedule: Schedule)                                                 extends Schedule
    final case class Combine(self: Schedule, other: Schedule, operator: Combine.Operator)   extends Schedule
    object Combine {
      sealed trait Operator
      object Operator {
        case object And extends Operator
        case object Or  extends Operator
      }
    }
    def and(self: Schedule, other: Schedule): Schedule = Combine(self, other, Combine.Operator.And)

    def or(self: Schedule, other: Schedule): Schedule = Combine(self, other, Combine.Operator.Or)
  }
}
