// #define STATIC_BIND

using System.Collections;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System;
using UnityEngine;
using FlatBuffers;
using UnityEngine.EventSystems;

/**
 * Provide user friendly access to native rust code
 */
namespace core
{
    public interface EventHandler
    {
        void AddSector(UInt32 id, space_data.V2 fromPos);
        void AddJump(UInt32 id, UInt32 fromSectorId, space_data.V2 fromPos, UInt32 toSectorId, space_data.V2 toPos);
        void AddObj(UInt32 id, space_data.EntityKind kind);
        void ObjTeleport(UInt32 id, UInt32 sectorId, space_data.V2 pos);
        void ObjMove(UInt32 id, space_data.V2 pos);
        void ObjJump(UInt32 id, UInt32 sectorId, space_data.V2 pos);
        void ObjDock(UInt32 id, UInt32 targetId);
        void ObjUndock(UInt32 id, UInt32 sectorId, space_data.V2 pos);
    }

#if STATIC_BIND
    static class Native
    {
        [DllImport("ffi_space.so", CharSet = CharSet.Unicode)]
        internal static extern CoreHandler space_domain_init_context(string args);
        [DllImport("ffi_space.so", CharSet = CharSet.Unicode)]
        internal static extern UInt32 space_domain_run_tick(CoreHandler ptr, UInt32 delta);
        [DllImport("ffi_space.so", CharSet = CharSet.Unicode)]
        internal static extern void space_domain_close_context(IntPtr ptr);
        [DllImport("ffi_space.so")]
        internal static extern UInt32 space_domain_set_data(CoreHandler ptr, UInt16 kind, byte[] buffer, UInt32 len);
        [DllImport("ffi_space.so", CharSet = CharSet.Unicode)]
        internal static extern UInt32 space_domain_get_data(CoreHandler ptr, Action<UInt16, IntPtr, UInt32> callback);
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
            Native.space_domain_close_context(handle);
            return true;
        }
    }
#endif

    public class Core : IDisposable
    {
        #if STATIC_BIND
        private CoreHandler coreHandler;

        #else
        [DllImport("__Internal")]
        public static extern IntPtr dlopen(string path, int flag);
        
        [DllImport("__Internal")]
        public static extern IntPtr dlerror();
 
        [DllImport("__Internal")]
        public static extern IntPtr dlsym(IntPtr handle, string symbolName);
 
        [DllImport("__Internal")]
        public static extern int dlclose(IntPtr handle);

        public static IntPtr LoadLibrary(string path)
        {
            Debug.Log($"Loading native library at {path}");
            IntPtr handle = dlopen(path, 2); // 1 lazy, 2 now, 0 ??
            if (handle == IntPtr.Zero)
            {
                var error = GetDLError();
                throw new Exception("Couldn't open native library at [" + path + "]: " + error);
            }

            return handle;
        }

        private static string GetDLError()
        {
            var errPtr = dlerror();

            if (errPtr == IntPtr.Zero)
            {
                return "Null pointer was returned from dlerror.";
            }
            else
            {
                return Marshal.PtrToStringAnsi(errPtr);
            }
        }

        public static void CloseLibrary(IntPtr libraryHandle)
        {
            if (libraryHandle != IntPtr.Zero)
            {
                Debug.Log("Closing native library");
                dlclose(libraryHandle);
            }
        } 
        
        public static T GetLibraryFunction<T>(
            IntPtr libraryHandle,
            string functionName) where T : class
        {
            // Debug.Log("Load native function "+functionName);
            
            IntPtr symbol = dlsym(libraryHandle, functionName);
            if (symbol == IntPtr.Zero)
            {
                var error = GetDLError();
                throw new Exception("Couldn't get function: " + functionName + ": "+ error);
            }
            return Marshal.GetDelegateForFunctionPointer(
                symbol,
                typeof(T)) as T;
        }
        
        delegate IntPtr CreateContext(string args);
        delegate void ContextClose(IntPtr ptr);
        delegate UInt32 ContextPush(IntPtr ptr, UInt16 kind, byte[] buffer, UInt32 len);
        delegate UInt32 ContextTake(IntPtr ptr, Action<UInt16, IntPtr, UInt32> callback);
        delegate UInt32 ContextTick(IntPtr ptr, UInt32 delta);

        private ContextClose nativeContextClose;
        private ContextPush nativeContextPush;
        private ContextTake nativeContextTake;
        private ContextTick nativeContextTick;
        
        private IntPtr libraryHandle;
        private IntPtr contextHandle;
        #endif

        public EventHandler eventHandler;

        public Core(string args, EventHandler eventHandler)
        {
            this.eventHandler = eventHandler;

            #if STATIC_BIND
            this.coreHandler = Native.space_domain_init_context(args);
            #else
            var libName = "libffi_space.so";
            var dataPath = Application.dataPath + "/Plugins/" + libName;
            libraryHandle = LoadLibrary(dataPath);

            // load methods
            this.nativeContextClose = GetLibraryFunction<ContextClose>(libraryHandle, "space_domain_close_context");
            this.nativeContextPush = GetLibraryFunction<ContextPush>(libraryHandle, "space_domain_set_data");
            this.nativeContextTake = GetLibraryFunction<ContextTake>(libraryHandle, "space_domain_get_data");
            this.nativeContextTick = GetLibraryFunction<ContextTick>(libraryHandle, "space_domain_run_tick");
            
            // start ffi context
            CreateContext createContext = GetLibraryFunction<CreateContext>(libraryHandle, "space_domain_init_context");
            contextHandle = createContext.Invoke(args);
            #endif
            Debug.Log("core initialize");
        }

        public void Dispose()
        {
            #if STATIC_BIND
            coreHandler.Dispose();
            #else
                        // close ffi context
            if (contextHandle != IntPtr.Zero && this.nativeContextClose != null)
            {
                this.nativeContextClose.Invoke(contextHandle);
                contextHandle = IntPtr.Zero;
            }

            // unload library
            CloseLibrary(libraryHandle);
            libraryHandle = IntPtr.Zero;
            #endif

            Debug.Log("core disposed");
        }

        public void Update(float delta) 
        {
            var delta_u32 = Convert.ToUInt32((int) Math.Floor(delta * 1000));
            #if STATIC_BIND
            var result = Native.space_domain_run_tick(coreHandler, delta_u32);
            #else
            var result = this.nativeContextTick(this.contextHandle, delta_u32);
            #endif

            if (result != 0)
            {
                throw new Exception("invalid result " + result);
            }
        }

        public void GetData()
        {
            if (this.eventHandler == null) throw new NullReferenceException("Event handler can not be null");

            byte[] bytes = null;

            #if STATIC_BIND
            var result = Native.space_domain_get_data(this.coreHandler, (kind, ptr, length) =>
            {
                // TODO: why moving parser and call to handler here crash rust with already borrow?
                bytes = ToByteArray(ptr, length);
            });
            #else
            var result = this.nativeContextTake(this.contextHandle, (kind, ptr, length) =>
            {
                bytes = ToByteArray(ptr, length);
            });
            #endif

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
                this.eventHandler.AddSector(entity.Id, entity.Coords);
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