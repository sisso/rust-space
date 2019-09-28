using System;
using System.Collections;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using UnityEngine;

static internal class CheckFFINative
{
    [DllImport("libspace.so", CharSet = CharSet.Unicode)]
    internal static extern Int32 add_numbers(Int32 a, Int32 b);

    [DllImport("libspace.so", CharSet = CharSet.Unicode)]
    internal static extern void execute();
}

public class CheckFFI : MonoBehaviour
{
    void Start()
    {
        var result = CheckFFINative.add_numbers(1, 3);
        Debug.Log("FFI working: " + result);

        CheckFFINative.execute();
        Debug.Log("done");
    }
}
