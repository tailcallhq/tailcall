package tailcall.gateway.internal

import zio.Chunk

object ChunkUtil:
  // scalafmt: { maxColumn = 1200 }
  def toTuple(f: Chunk[Any]): Product =
    f.length match
      case 2  => Tuple2(f(0), f(1))
      case 3  => Tuple3(f(0), f(1), f(2))
      case 4  => Tuple4(f(0), f(1), f(2), f(3))
      case 5  => Tuple5(f(0), f(1), f(2), f(3), f(4))
      case 6  => Tuple6(f(0), f(1), f(2), f(3), f(4), f(5))
      case 7  => Tuple7(f(0), f(1), f(2), f(3), f(4), f(5), f(6))
      case 8  => Tuple8(f(0), f(1), f(2), f(3), f(4), f(5), f(6), f(7))
      case 9  => Tuple9(f(0), f(1), f(2), f(3), f(4), f(5), f(6), f(7), f(8))
      case 10 => Tuple10(f(0), f(1), f(2), f(3), f(4), f(5), f(6), f(7), f(8), f(9))
      case 11 => Tuple11(f(0), f(1), f(2), f(3), f(4), f(5), f(6), f(7), f(8), f(9), f(10))
      case 12 => Tuple12(f(0), f(1), f(2), f(3), f(4), f(5), f(6), f(7), f(8), f(9), f(10), f(11))
      case 13 => Tuple13(f(0), f(1), f(2), f(3), f(4), f(5), f(6), f(7), f(8), f(9), f(10), f(11), f(12))
      case 14 => Tuple14(f(0), f(1), f(2), f(3), f(4), f(5), f(6), f(7), f(8), f(9), f(10), f(11), f(12), f(13))
      case 15 => Tuple15(f(0), f(1), f(2), f(3), f(4), f(5), f(6), f(7), f(8), f(9), f(10), f(11), f(12), f(13), f(14))
      case 16 => Tuple16(f(0), f(1), f(2), f(3), f(4), f(5), f(6), f(7), f(8), f(9), f(10), f(11), f(12), f(13), f(14), f(15))
      case 17 => Tuple17(f(0), f(1), f(2), f(3), f(4), f(5), f(6), f(7), f(8), f(9), f(10), f(11), f(12), f(13), f(14), f(15), f(16))
      case 18 => Tuple18(f(0), f(1), f(2), f(3), f(4), f(5), f(6), f(7), f(8), f(9), f(10), f(11), f(12), f(13), f(14), f(15), f(16), f(17))
      case 19 => Tuple19(f(0), f(1), f(2), f(3), f(4), f(5), f(6), f(7), f(8), f(9), f(10), f(11), f(12), f(13), f(14), f(15), f(16), f(17), f(18))
      case 20 => Tuple20(f(0), f(1), f(2), f(3), f(4), f(5), f(6), f(7), f(8), f(9), f(10), f(11), f(12), f(13), f(14), f(15), f(16), f(17), f(18), f(19))
      case 21 => Tuple21(f(0), f(1), f(2), f(3), f(4), f(5), f(6), f(7), f(8), f(9), f(10), f(11), f(12), f(13), f(14), f(15), f(16), f(17), f(18), f(19), f(20))
      case 22 => Tuple22(f(0), f(1), f(2), f(3), f(4), f(5), f(6), f(7), f(8), f(9), f(10), f(11), f(12), f(13), f(14), f(15), f(16), f(17), f(18), f(19), f(20), f(21))
      case _  => null
