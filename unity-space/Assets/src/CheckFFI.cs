using System;
using System.Collections;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using UnityEngine;

static class CheckFFINative
{
    [DllImport("libspace.so", CharSet = CharSet.Unicode)]
    internal static extern Int32 add_numbers(Int32 a, Int32 b);

    [DllImport("libspace.so", CharSet = CharSet.Unicode)]
    internal static extern void execute();

    public static void Execute()
    {
        var result = CheckFFINative.add_numbers(1, 3);
        Debug.Log("FFI working: " + result);

        CheckFFINative.execute();
        Debug.Log("done");

        throw new Exception("WTF");
    }
}

public class CheckFFI : MonoBehaviour
{
    void Start()
    {
        CheckFFINative.Execute();
    }
}
