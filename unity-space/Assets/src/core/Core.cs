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
        internal static extern void run_tick(CoreHandler ptr, UInt32 delta);
        [DllImport("libspace.so", CharSet = CharSet.Unicode)]
        internal static extern void close(CoreHandler ptr);
        [DllImport("libspace.so")]
        internal static extern bool set_data(CoreHandler ptr, byte[] buffer, UInt32 len);
        [DllImport("libspace.so", CharSet = CharSet.Unicode)]
        internal static extern bool get_data(CoreHandler ptr, Action<IntPtr, UInt32> callback);
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
            Native.close(this);
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

        public void Update(float delta) 
        {
            var delta_u32 = (UInt32) Math.Floor(delta * 1000);
            Native.run_tick(handler, delta_u32);
        }

        public void GetData()
        {
            byte[] bytes = null;

            Native.get_data(this.handler, (ptr, length) =>
            {
                bytes = ToByteArray(ptr, length);
            });

            if (bytes == null)
            {
                throw new Exception("Null bytes returned from Native");
            }

            Debug.Log("receive " + bytes.Length + " bytes");
        }

        private static byte[] ToByteArray(IntPtr ptr, uint length)
        {
            int len = Convert.ToInt32(length);
            var bytes = new byte[len];
            Marshal.Copy(ptr, bytes, 0, len);
            return bytes;
        }    
    }       
}