using System;
using System.Runtime.InteropServices;

static class FFI
{
    [DllImport("../../../rust/target/debug/libspace.so", CharSet = CharSet.Unicode)]
    internal static extern Int32 add_numbers(Int32 a, Int32 b);

    [DllImport("../../../rust/target/debug/libspace.so", CharSet = CharSet.Unicode)]
    internal static extern void execute();
}

class MainClass
{
    public static void Main(string[] args)
    {
        var result = FFI.add_numbers(1, 3);
        Console.WriteLine("FFI working: "+result);

        FFI.execute();
        Console.WriteLine("done");
    }
}
