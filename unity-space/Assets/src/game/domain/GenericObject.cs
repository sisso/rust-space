using System;
using System.Collections;
using System.Collections.Generic;
using UnityEngine;

namespace game.domain
{

    public class GenericObject : MonoBehaviour
    {
        public Id id;
        public ObjKind kind;

        public void UpdateName()
        {
            var kindName = Enum.GetName(typeof(ObjKind), this.kind);
            gameObject.name = kindName + " (" + id.value + ")";
        }
    }
}