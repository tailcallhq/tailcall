import os
import shutil

print("Running python script...")
os.environ["PROTOC"] = shutil.which("protoc")
print("{}".format(os.environ["PROTOC"]))