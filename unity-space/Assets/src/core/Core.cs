using System.Collections;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System;
using UnityEngine;
using FlatBuffers;

namespace core
{

    static class Native
    {
        [DllImport("libspace.so", CharSet = CharSet.Unicode)]
        internal static extern CoreHandler init(string args);
        [DllImport("libspace.so", CharSet = CharSet.Unicode)]
        internal static extern UInt32 run_tick(CoreHandler ptr, UInt32 delta);
        [DllImport("libspace.so", CharSet = CharSet.Unicode)]
        internal static extern void close(IntPtr ptr);
        [DllImport("libspace.so")]
        internal static extern UInt32 set_data(CoreHandler ptr, byte[] buffer, UInt32 len);
        [DllImport("libspace.so", CharSet = CharSet.Unicode)]
        internal static extern UInt32 get_data(CoreHandler ptr, Action<IntPtr, UInt32> callback);
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
            Debug.Log("core disposed");
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

            // unmarshlar
            var buffer = new ByteBuffer(bytes);
            var outputs = space_data.Outputs.GetRootAsOutputs(buffer);

            for (int i = 0; i < outputs.SectorsLength; i++)
            {
                var entity = outputs.Sectors(i) ?? throw new NullReferenceException();
                var id = entity.Id;
                Debug.Log("adding sector "+id);
            }

            for (int i = 0; i < outputs.JumpsLength; i++)
            {
                var entity = outputs.Jumps(i) ?? throw new NullReferenceException();
                var id = entity.Id;
                var pos = new Vector2(entity.Pos.X, entity.Pos.Y);
                var sector = (int) entity.SectorId;
                var toSector = (int) entity.ToSectorId;
                var toPos = new Vector2(entity.ToPos.X, entity.ToPos.Y);
                Debug.Log("adding jump gate" + id + " from " + pos + "/" + sector + " to " + toSector + "/" + toPos);
            }

            for (int i = 0; i < outputs.EntitiesNewLength; i++)
            {
                var entity = outputs.EntitiesNew(i) ?? throw new NullReferenceException();
                var id = entity.Id;
                var kind = entity.Kind;
                var sectorId = (int)entity.SectorId;
                var pos = new Vector2(entity.Pos.X, entity.Pos.Y);
                Debug.Log("adding " + id + " of type " + kind + " at " + pos + "/" + sectorId);
            }

            for (int i = 0; i < outputs.EntitiesMoveLength; i++)
            {
                var entity = outputs.EntitiesMove(i) ?? throw new NullReferenceException();
                var id = entity.Id;
                var pos = new Vector2(entity.Pos.X, entity.Pos.Y);
                Debug.Log("moved " + id + " to " + pos);
            }

            for (int i = 0; i < outputs.EntitiesJumpLength; i++)
            {
                var entity = outputs.EntitiesJump(i) ?? throw new NullReferenceException();
                var id = entity.Id;
                var pos = new Vector2(entity.Pos.X, entity.Pos.Y);
                var sector = (int) entity.SectorId;
                Debug.Log("jump " + id + " to " + pos + "/" + sector);
            }
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