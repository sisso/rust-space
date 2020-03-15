using System.Collections;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System;
using UnityEngine;

namespace core
{

    static class Native
    {
        [DllImport("libspace.so", CharSet = CharSet.Unicode)]
        internal static extern CoreHandler init(string args);
        [DllImport("libspace.so", CharSet = CharSet.Unicode)]
        internal static extern void close(IntPtr ptr);
    }

    internal class CoreHandler : SafeHandle
    {
        public CoreHandler() : base(IntPtr.Zero, true)
        {
        }

        public override bool IsInvalid
        {
            get
            {
                return false;
            }
        }

        protected override bool ReleaseHandle()
        {
            Native.close(handle);
            return true;
        }
    }

    public class Core : IDisposable
    {
        private CoreHandler handler;

        public Core(string args)
        {
            this.handler = Native.init(args);
            Debug.Log("core initialize");
        }

        public void Dispose()
        {
            handler.Dispose();
        }
    }       
}