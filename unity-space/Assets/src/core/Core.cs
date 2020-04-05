using System.Collections;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System;
using UnityEngine;
using FlatBuffers;

/**
 * Provide user friendly access to native rust code
 */
namespace core
{
    public interface EventHandler
    {
        void AddSector(UInt32 id);
        void AddJump(UInt32 id, UInt32 fromSectorId, space_data.V2 fromPos, UInt32 toSectorId, space_data.V2 toPos);
        void AddObj(UInt32 id, space_data.EntityKind kind);
        void ObjTeleport(UInt32 id, UInt32 sectorId, space_data.V2 pos);
        void ObjMove(UInt32 id, space_data.V2 pos);
        void ObjJump(UInt32 id, UInt32 sectorId, space_data.V2 pos);
        void ObjDock(UInt32 id, UInt32 targetId);
        void ObjUndock(UInt32 id, UInt32 sectorId, space_data.V2 pos);
    }

    static class Native
    {
        [DllImport("libspace.so", CharSet = CharSet.Unicode)]
        internal static extern CoreHandler init_context(string args);
        [DllImport("libspace.so", CharSet = CharSet.Unicode)]
        internal static extern UInt32 run_tick(CoreHandler ptr, UInt32 delta);
        [DllImport("libspace.so", CharSet = CharSet.Unicode)]
        internal static extern void close_context(IntPtr ptr);
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
            Native.close_context(handle);
            return true;
        }
    }

    public class Core : IDisposable
    {
        private CoreHandler coreHandler;

        public EventHandler eventHandler;

        public Core(string args, EventHandler eventHandler)
        {
            this.coreHandler = Native.init_context(args);
            this.eventHandler = eventHandler;
            Debug.Log("core initialize");
        }

        public void Dispose()
        {
            coreHandler.Dispose();
            Debug.Log("core disposed");
        }

        public void Update(float delta) 
        {
            var delta_u32 = (UInt32) Math.Floor(delta * 1000);
            var result = Native.run_tick(coreHandler, delta_u32);

            if (result != 0)
            {
                throw new Exception("invalid result " + result);
            }
        }

        public void GetData()
        {
            if (this.eventHandler == null) throw new NullReferenceException("Event handler can not be null");

            byte[] bytes = null;

            var result = Native.get_data(this.coreHandler, (ptr, length) =>
            {
                // TODO: why moving parser and call to handler here crash rust with already borrow?
                bytes = ToByteArray(ptr, length);
            });

            if (result != 0)
            {
                throw new Exception("invalid result " + result);
            }
            if (bytes == null)
            {
                throw new Exception("Null bytes returned from Native");
            }

            var buffer = new ByteBuffer(bytes);
            var outputs = space_data.Outputs.GetRootAsOutputs(buffer);

            for (int i = 0; i < outputs.SectorsLength; i++)
            {
                var entity = outputs.Sectors(i) ?? throw new NullReferenceException();
                this.eventHandler.AddSector(entity.Id);
            }

            for (int i = 0; i < outputs.JumpsLength; i++)
            {
                var entity = outputs.Jumps(i) ?? throw new NullReferenceException();
                this.eventHandler.AddJump(entity.Id, entity.SectorId, entity.Pos, entity.ToSectorId, entity.ToPos);
            }

            for (int i = 0; i < outputs.EntitiesNewLength; i++)
            {
                var entity = outputs.EntitiesNew(i) ?? throw new NullReferenceException();
                this.eventHandler.AddObj(entity.Id, entity.Kind);
            }

            for (int i = 0; i < outputs.EntitiesTeleportLength; i++)
            {
                var entity = outputs.EntitiesTeleport(i) ?? throw new NullReferenceException();
                this.eventHandler.ObjTeleport(entity.Id, entity.SectorId, entity.Pos);
            }

            for (int i = 0; i < outputs.EntitiesMoveLength; i++)
            {
                var entity = outputs.EntitiesMove(i) ?? throw new NullReferenceException();
                this.eventHandler.ObjMove(entity.Id, entity.Pos);
            }

            for (int i = 0; i < outputs.EntitiesJumpLength; i++)
            {
                var entity = outputs.EntitiesJump(i) ?? throw new NullReferenceException();
                this.eventHandler.ObjJump(entity.Id, entity.SectorId, entity.Pos);
            }

            for (int i = 0; i < outputs.EntitiesDockLength; i++)
            {
                var entity = outputs.EntitiesDock(i) ?? throw new NullReferenceException();
                this.eventHandler.ObjDock(entity.Id, entity.TargetId);
            }

            for (int i = 0; i < outputs.EntitiesUndockLength; i++)
            {
                var entity = outputs.EntitiesUndock(i) ?? throw new NullReferenceException();
                this.eventHandler.ObjUndock(entity.Id, entity.SectorId, entity.Pos);
            }
        }

        private static byte[] ToByteArray(IntPtr ptr, uint length)
        {
            // TODO: remove copy
            int len = Convert.ToInt32(length);
            var bytes = new byte[len];
            Marshal.Copy(ptr, bytes, 0, len);
            return bytes;
        }
    }
}