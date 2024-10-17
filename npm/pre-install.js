// @ts-check
import {familySync, GLIBC, MUSL} from "detect-libc"
import get_matched_platform from "./utils.js"

const os = process.platform
const arch = process.arch
const libcFamily = familySync()

let libc = ""
if (os === "win32") {
  libc = "msvc"
} else {
  libc = libcFamily === GLIBC ? "gnu" : libcFamily === MUSL ? "musl" : ""
}

const matched_platform = get_matched_platform(os, arch, libc)

if (matched_platform == null) {
  const redColor = "\x1b[31m"
  const resetColor = "\x1b[0m"
  console.error(`${redColor} Tailcall does not support platform - ${os}, arch - ${arch}, libc - ${libc} ${resetColor}`)
  process.exit(1)
}
